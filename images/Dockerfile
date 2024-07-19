FROM python:3.12.4-alpine3.20
WORKDIR /app
COPY . .
RUN apk update && apk add --no-cache \
    ca-certificates \
    && update-ca-certificates
RUN pip install --upgrade pip
RUN pip install -r requirements.txt
CMD ["python3", "main.py"]

FROM python:3.12.4-alpine3.20
WORKDIR /app
COPY . .
RUN apk update && apk add --no-cache \
    ca-certificates \
    && update-ca-certificates
RUN pip install --upgrade pip
RUN pip install -r requirements.txt
CMD ["python3", "main.py"]
