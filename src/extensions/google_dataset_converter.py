import argparse
import csv
import gzip
import json
import os
import random


def read_nodes(path, max_nodes):
    machine_events = os.path.join(path, "machine_events")
    machine_files = [os.path.join(machine_events, f) for f in os.listdir(machine_events)
                     if os.path.isfile(os.path.join(machine_events, f))]
    machine_files.sort()

    nodes = list()
    for machine_file in machine_files:
        break_flag = False
        with gzip.open(machine_file, 'rt') as file:
            reader = csv.reader(file)
            for row in reader:
                if float(row[0]) != 0:
                    break_flag = True
                    break
                if '' in [row[4], row[5]]:
                    continue

                cpu = float(row[4])
                memory = float(row[5])
                nodes.append((cpu, memory))
            if break_flag:
                break

    random.shuffle(nodes)
    return nodes[:min(max_nodes, len(nodes))]


def read_pod_ids(path, max_pods, time_in_days):
    task_events = os.path.join(path, "task_events")
    event_files = [os.path.join(task_events, f) for f in os.listdir(task_events)
                   if os.path.isfile(os.path.join(task_events, f))]
    event_files.sort()

    pod_ids = set()
    for event_file in event_files:
        break_flag = False
        with gzip.open(event_file, 'rt') as file:
            reader = csv.reader(file)
            for row in reader:
                if float(row[0]) / (24 * 60 * 60 * 1000000) > time_in_days:
                    break_flag = True
                    break
                if '' in [row[2], row[3], row[8], row[9], row[10]]:
                    continue
                if int(row[5]) == 0:
                    pod_ids.add((row[2], row[3]))
                elif int(row[5]) in [2, 3, 4, 5, 6]:
                    pod_ids.discard((row[2], row[3]))
            if break_flag:
                break

    pod_ids = list(pod_ids)
    random.shuffle(pod_ids)
    return set(pod_ids[:min(max_pods, len(pod_ids))])


def read_pods(path, pod_ids, time_in_days):
    task_events = os.path.join(path, "task_events")
    event_files = [os.path.join(task_events, f) for f in os.listdir(task_events)
                   if os.path.isfile(os.path.join(task_events, f))]
    event_files.sort()

    pods = dict()
    for event_file in event_files:
        print(event_file)
        break_flag = False
        with gzip.open(event_file, 'rt') as file:
            reader = csv.reader(file)
            for row in reader:
                if float(row[0]) / (24 * 60 * 60 * 1000000) > time_in_days:
                    break_flag = True
                    break
                if (row[2], row[3]) not in pod_ids:
                    continue
                if '' in [row[2], row[3], row[8], row[9], row[10]]:
                    continue
                if int(row[5]) != 0:
                    continue
                pods[(row[2], row[3])] = {
                    "timestamp": float(row[0]) / 1000000,
                    "requested_cpu": float(row[9]),
                    "limit_cpu": float(row[9]),
                    "requested_memory": float(row[10]),
                    "limit_memory": float(row[10]),
                    "priority_weight": int(row[8]),
                    "cpu_load_model": {
                        "type": "CONST",
                        "value": float(row[9])
                    },
                    "memory_load_model": {
                        "type": "CONST",
                        "value": float(row[10])
                    }
                }
            if break_flag:
                break

    return pods


def fill_usage_models(path, measurement_period, time_in_days, pods):
    task_usage = os.path.join(path, "task_usage")
    usage_files = [os.path.join(task_usage, f) for f in os.listdir(task_usage)
                   if os.path.isfile(os.path.join(task_usage, f))]
    usage_files.sort()

    cpu_trace = dict()
    memory_trace = dict()
    for usage_file in usage_files:
        print(usage_file)
        break_flag = False
        with gzip.open(usage_file, 'rt') as file:
            reader = csv.reader(file)
            for row in reader:
                pod_id = (row[2], row[3])
                timestamp = float(row[0]) / 1000000
                if timestamp / (24 * 60 * 60) > time_in_days:
                    break_flag = True
                    break
                if pod_id not in pods:
                    continue
                if timestamp < pods[pod_id]['timestamp']:
                    continue
                if '' in [row[5], row[6]]:
                    continue

                timestamp = timestamp - pods[pod_id]["timestamp"]

                if pod_id not in cpu_trace:
                    cpu_trace[pod_id] = list()
                    memory_trace[pod_id] = list()
                elif cpu_trace[pod_id][-1]['timestamp'] + measurement_period > timestamp:
                    continue

                cpu_trace[pod_id].append({
                    "timestamp": timestamp,
                    "value": float(row[5])
                })
                memory_trace[pod_id].append({
                    "timestamp": timestamp,
                    "value": float(row[6])
                })
            if break_flag:
                break

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


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument('--input', type=str, help='Path of directory with google dataset (2011)')
    parser.add_argument('--output', type=str, help='Path of file with result')
    parser.add_argument('--nodes', type=int, help='Max count of cluster nodes (about 15k in original cluster)')
    parser.add_argument('--pods', type=int, help='Max count of pods')
    parser.add_argument('--time', type=float, help='Time in days (from start of the dataset)')
    parser.add_argument('--mperiod', type=float, help='Period of measurement of pod\'s resources (in '
                                                                 'seconds)')
    parser.add_argument('--deployments', type=float, help='Part of deployments (from all pod\'s count)')
    args = parser.parse_args()

    print("read nodes")
    nodes = read_nodes(args.input, args.nodes)
    print("read task events")
    pods_ids = read_pod_ids(args.input, args.pods, args.time)
    pods = read_pods(args.input, pods_ids, args.time)
    print("read task usage")
    fill_usage_models(args.input, args.mperiod, args.time, pods)
    print("finish read")

    output_trace = list()
    for node in nodes:
        output_trace.append({
            "type": "ADD_NODE",
            "cpu": node[0],
            "memory": node[1]
        })
    for pod_id in pods:
        pod = pods[pod_id]
        if random.random() < args.deployments:
            pod["type"] = "SUBMIT_DEPLOYMENT"
            pod["cnt_replicas"] = 1
        else:
            pod["type"] = "SUBMIT_POD"
        output_trace.append(pod)
    with open("google_trace.json", "w+") as output_file:
        json.dump(output_trace, output_file)
