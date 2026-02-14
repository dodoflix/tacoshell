FROM golang:1.20-alpine AS builder

WORKDIR /app
COPY . .
RUN go build -o tacoshell cmd/tacoshell/main.go

FROM alpine:latest
WORKDIR /root/
COPY --from=builder /app/tacoshell .
CMD ["./tacoshell"]

