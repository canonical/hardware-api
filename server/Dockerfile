FROM python:3.13

WORKDIR /home/server

COPY --from=ghcr.io/astral-sh/uv:0.11.18 /uv /uvx /bin/

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

EXPOSE 8080

CMD ["uv", "run", "uvicorn", "hwapi.main:app", "--host", "0.0.0.0", "--port", "8080"]
