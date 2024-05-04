import argparse
import csv
import json
import os.path

if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument('--input', type=str, help='Path of directory with alibaba dataset (2017)')
    parser.add_argument('--output', type=str, help='Path of file with result')
    args = parser.parse_args()

    nodes = list()
    with open(os.path.join(args.input, "server_event.csv")) as nodes_file:
        reader = csv.reader(nodes_file)
        for row in reader:
            if row[2] != "add":
                continue
            cpu = float(row[4])
            memory = float(row[5])
            nodes.append((cpu, memory))

    pods = dict()
    with open(os.path.join(args.input, "container_event.csv")) as pods_file:
        reader = csv.reader(pods_file)
        for row in reader:
            if row[1] != "Create":
                continue
            pods[row[2]] = {
                "timestamp": float(row[0]),
                "requested_cpu": float(row[4]),
                "limit_cpu": float(row[4]),
                "requested_memory": float(row[5]),
                "limit_memory": float(row[5]),
                "priority_weight": 1,
                "cpu_load_model": {
                    "type": "CONST",
                    "value": float(row[4])
                },
                "memory_load_model": {
                    "type": "CONST",
                    "value": float(row[5])
                }
            }

    cpu_trace = dict()
    memory_trace = dict()
    with open(os.path.join(args.input, "container_usage.csv")) as resource_trace_file:
        reader = csv.reader(resource_trace_file)
        for row in reader:
            pod_id = row[1]
            if pod_id not in pods:
                continue
            if row[2] == '' or row[3] == '':
                continue

            if pod_id not in cpu_trace:
                cpu_trace[pod_id] = list()
                memory_trace[pod_id] = list()

            requested_cpu = pods[pod_id]["requested_cpu"]
            requested_memory = pods[pod_id]["requested_memory"]

            timestamp = float(row[0]) - pods[pod_id]["timestamp"]
            cpu_trace[pod_id].append({
                "timestamp": timestamp,
                "value": float(row[2]) / 100 * requested_cpu
            })
            memory_trace[pod_id].append({
                "timestamp": timestamp,
                "value": float(row[3]) / 100 * requested_memory
            })

    for pod_id in cpu_trace:
        cpu_trace[pod_id].sort(key=lambda x: x["timestamp"])
        memory_trace[pod_id].sort(key=lambda x: x["timestamp"])

        pods[pod_id]["cpu_load_model"] = {
            "type": "TRACE",
            "snapshots": cpu_trace[pod_id]
        }

        pods[pod_id]["memory_load_model"] = {
            "type": "TRACE",
            "snapshots": memory_trace[pod_id]
        }

    output_trace = list()
    for node in nodes:
        output_trace.append({
            "type": "ADD_NODE",
            "cpu": node[0],
            "memory": node[1]
        })

    for pod_id in pods:
        pod = pods[pod_id]
        pod["type"] = "SUBMIT_POD"
        output_trace.append(pod)

    with open(args.output, "w+") as output_file:
        json.dump(output_trace, output_file)
