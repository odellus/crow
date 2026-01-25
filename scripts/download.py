-e import requests
import os
base_dir = \"docs/openhands_book\"
llms_file = os.path.join(base_dir, \"llms.txt\")
with open(llms_file, \"r\") as f:
    for line in f:
        start = line.find(\"(\")
        end = line.find(\")\")
        if start != -1 and end != -1:
            url = line[start+1:end]
            try:
                resp = requests.get(url)
                if resp.status_code == 200:
                    clean_url = url.replace(\"https://docs.openhands.dev/\", \"\")
                    local_path = os.path.join(base_dir, clean_url)
                    os.makedirs(os.path.dirname(local_path), exist_ok=True)
                    with open(local_path, \"w\", encoding=\"utf-8\") as f_out:
                        f_out.write(resp.text)
            except Exception as e:
                print(f\"Error: {e}\")
