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

CMD ["uvicorn", "hwapi.main:app", "--host", "0.0.0.0", "--port", "8080"]
