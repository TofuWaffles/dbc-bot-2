FROM python:3.12.4-alpine3.20
WORKDIR /app
COPY requirements.txt .
RUN apk update && apk add --no-cache \
    ca-certificates \
    && update-ca-certificates
RUN --mount=type=cache,target=/root/.cache \
    pip install -r requirements.txt
COPY . .
CMD ["python3", "main.py"]
