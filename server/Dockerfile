FROM ubuntu:24.04
COPY --from=ghcr.io/astral-sh/uv:0.11.18 /uv /uvx /bin/

RUN --mount=type=cache,target=/var/cache/apt \
    --mount=type=cache,target=/var/lib/apt,sharing=locked \
    apt-get update -y && \
    DEBIAN_FRONTEND=noninteractive apt-get install --no-install-recommends --yes \
    python3-venv \
    sqlite3

WORKDIR /home/server

ENV UV_COMPILE_BYTECODE=1
ENV UV_LINK_MODE=copy

RUN --mount=type=cache,target=/root/.cache/uv \
    --mount=type=bind,source=uv.lock,target=uv.lock \
    --mount=type=bind,source=pyproject.toml,target=pyproject.toml \
    uv sync --locked --no-install-project

COPY . /home/server
RUN --mount=type=cache,target=/root/.cache/uv \
    uv sync --locked

ENV PATH="/home/server/.venv/bin:$PATH"

ARG IMPORT_TOOL_PATH=./scripts/import_from_c3.py
ARG DB_URL=sqlite:////home/server/hwapi.db
ENV DB_URL=$DB_URL
ARG C3_URL=https://certification.canonical.com
ENV C3_URL=$C3_URL

RUN if [ -x $IMPORT_TOOL_PATH ]; then \
    uv run $IMPORT_TOOL_PATH; \
    fi

EXPOSE 8080

CMD ["uvicorn", "hwapi.main:app", "--host", "0.0.0.0", "--port", "8080"]
