#!/usr/bin/env python3
"""Generates deterministic synthetic Nginx/Apache HTTP access logs with latency."""
import argparse
from pathlib import Path
import random

IPS = [
    "192.168.1.10", "10.0.0.42", "172.16.0.5", "203.0.113.88",
    "198.51.100.14", "192.0.2.199", "10.24.12.9", "172.31.255.1",
]
METHODS = ["GET", "POST", "PUT", "DELETE"]
PATHS = [
    "/api/v1/users", "/api/v1/orders", "/api/v1/checkout",
    "/healthz", "/static/app.js", "/login", "/api/v2/metrics",
]
STATUS_WEIGHTS = [
    (200, 0.75), (201, 0.08), (304, 0.05),
    (400, 0.04), (401, 0.03), (404, 0.03),
    (500, 0.015), (503, 0.005),
]


def generate_logs(output_path: Path, count: int, seed: int = 42) -> None:
    rng = random.Random(seed)
    statuses = [s for s, _ in STATUS_WEIGHTS]
    weights = [w for _, w in STATUS_WEIGHTS]

    output_path.parent.mkdir(parents=True, exist_ok=True)
    with output_path.open("w", encoding="utf-8") as f:
        for i in range(count):
            ip = rng.choice(IPS)
            method = rng.choice(METHODS)
            path = rng.choice(PATHS)
            status = rng.choices(statuses, weights=weights, k=1)[0]
            bytes_sent = rng.randint(64, 16384)
            # Inject tail latency spikes on 5xx responses
            latency_ms = rng.uniform(80.0, 950.0) if status >= 500 else rng.uniform(0.5, 45.0)
            sec = i % 60
            minute = (i // 60) % 60
            hour = (i // 3600) % 24
            line = (
                f'{ip} - - [25/Oct/2026:{hour:02d}:{minute:02d}:{sec:02d} +0000] '
                f'"{method} {path} HTTP/1.1" {status} {bytes_sent} {latency_ms:.2f}\n'
            )
            f.write(line)

    size_mb = output_path.stat().st_size / (1024 * 1024)
    print(f"[gen_logs] Wrote {count:,} lines ({size_mb:.2f} MB) to {output_path}")


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description="Generate synthetic HTTP access logs.")
    parser.add_argument("-n", "--lines", type=int, default=100_000, help="Number of log lines")
    parser.add_argument("-o", "--output", type=Path, default=Path("data/access.log"), help="Output file")
    args = parser.parse_args()
    generate_logs(args.output, args.lines)
  
