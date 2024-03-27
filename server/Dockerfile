FROM python:3.10

WORKDIR /home/app
EXPOSE 8080

ENV PYTHONPATH=/home/app
COPY poetry.lock .
COPY pyproject.toml .
RUN pip3 install poetry && \
    poetry config virtualenvs.create false && \
    poetry install --no-interaction --no-ansi

COPY . .

ENV DB_URL "sqlite:////home/app/data/hwapi.db"
# Don't populate existing DB with dummy data
RUN if ! [ -f data/hwapi.db ]; then \
    mkdir -p data; \
    python scripts/init_db.py; \
    fi

VOLUME ["/app/data"]

CMD ["uvicorn", "hwapi.main:app", "--host", "0.0.0.0", "--port", "8080", "--reload"]
