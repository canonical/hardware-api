FROM python:3.10

WORKDIR /home/code
EXPOSE 8080

ENV PYTHONPATH=/home/code
COPY poetry.lock .
COPY pyproject.toml .
RUN pip3 install poetry && \
    poetry config virtualenvs.create false && \
    poetry install --no-interaction --no-ansi

COPY . .

ARG IMPORT_TOOL_PATH=./scripts/import_from_c3.py
ARG DB_URL=sqlite:////home/code/hwapi.db
ENV DB_URL=$DB_URL
ARG C3_URL=https://certification.canonical.com
ENV C3_URL=$C3_URL

RUN if [ -x $IMPORT_TOOL_PATH ]; then \
        python3 $IMPORT_TOOL_PATH; \
    fi


CMD ["uvicorn", "hwapi.main:app", "--host", "0.0.0.0", "--port", "8080"]
