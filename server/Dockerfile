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

ARG DB_URL
ENV DB_URL=$DB_URL
ARG C3_URL
ENV C3_URL=$C3_URL

ARG SCRIPT_NAME=import_from_c3.py
RUN if [ -n "$SCRIPT_NAME" ]; then python scripts/$SCRIPT_NAME; fi

CMD ["uvicorn", "hwapi.main:app", "--host", "0.0.0.0", "--port", "8080"]
