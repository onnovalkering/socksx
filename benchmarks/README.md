# Benchmarks

## Setup

```shell
docker network create epi
docker run -dt --name nginx --net epi nginx
export SERVER_IP=$(docker inspect -f '{{range .NetworkSettings.Networks}}{{.IPAddress}}{{end}}' nginx)

```

## No proxy

```shell
docker run -t --rm --net epi --privileged onnovalkering/socksx-httping 60 1s $SERVER_IP
```

## SOCKS5 proxy
```shell
docker run -dt --rm --name proxy --net epi onnovalkering/socksx-proxy --socks 5
export PROXY_IP=$(docker inspect -f '{{range .NetworkSettings.Networks}}{{.IPAddress}}{{end}}' proxy)

docker run -t --rm --net epi --privileged onnovalkering/socksx-httping 60 1s $SERVER_IP $PROXY_IP "5"
```

## SOCKS6 proxy
```shell
docker run -dt --rm --name proxy --net epi onnovalkering/socksx-proxy --socks 6
export PROXY_IP=$(docker inspect -f '{{range .NetworkSettings.Networks}}{{.IPAddress}}{{end}}' proxy)

docker run -t --rm --net epi --privileged onnovalkering/socksx-httping 60 1s $SERVER_IP $PROXY_IP "6"
```
