# Build Docker images
build: build-httping build-iperf build-proxy build-wrk

build-httping:
	docker build -t onnovalkering/socksx-httping -f Dockerfile.httping ..

build-iperf:
	docker build -t onnovalkering/socksx-iperf -f Dockerfile.iperf ..

build-proxy:
	docker build -t onnovalkering/socksx-proxy -f Dockerfile.proxy ..

build-wrk:
	docker build -t onnovalkering/socksx-wrk -f Dockerfile.wrk ..

# Push Docker images
push: push-httping push-iperf push-proxy push-wrk

push-httping:
	docker push onnovalkering/socksx-httping

push-iperf:
	docker push onnovalkering/socksx-iperf

push-proxy:
	docker push onnovalkering/socksx-proxy

push-wrk:
	docker push onnovalkering/socksx-wrk
