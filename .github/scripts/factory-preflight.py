#!/usr/bin/env python3
"""Check existing CI credentials without revealing account data or secrets."""
import json
import os
import urllib.error
import urllib.request

account = os.environ["CLOUDFLARE_ACCOUNT_ID"]
token = os.environ["CLOUDFLARE_API_TOKEN"]
checks = {
    "Workers": f"accounts/{account}/workers/services",
    "Containers": f"accounts/{account}/containers/applications",
    "Workflows": f"accounts/{account}/workflows",
    "R2": f"accounts/{account}/r2/buckets",
    "AI Gateway": f"accounts/{account}/ai-gateway/gateways",
}
results = {}
for product, path in checks.items():
    request = urllib.request.Request(
        "https://api.cloudflare.com/client/v4/" + path,
        headers={"Authorization": "Bearer " + token},
    )
    try:
        with urllib.request.urlopen(request, timeout=30) as response:
            payload = json.load(response)
            status = response.status
    except urllib.error.HTTPError as error:
        status = error.code
        try:
            payload = json.load(error)
        except (ValueError, UnicodeDecodeError):
            payload = {}
    results[product] = {
        "http_status": status,
        "success": payload.get("success", False),
        "error_codes": [error.get("code") for error in (payload.get("errors") or [])],
    }
    print(product + ": " + json.dumps(results[product]))

with open(os.environ["GITHUB_STEP_SUMMARY"], "a") as summary:
    summary.write("Factory credential preflight (read-only; no resources changed)\n\n")
    summary.write("| Product | HTTP status |\n| --- | --- |\n")
    for product, result in results.items():
        summary.write(f"| {product} | {result['http_status']} |\n")
    summary.write("\nRead access does not establish deployment permissions.\n")
