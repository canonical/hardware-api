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

ARG DATA_LOADING_SCRIPT_FILE=import_from_c3.py
RUN if [ -n "$DATA_LOADING_SCRIPT_FILE" ]; then python scripts/$DATA_LOADING_SCRIPT_FILE; fi

CMD ["uvicorn", "hwapi.main:app", "--host", "0.0.0.0", "--port", "8080"]
