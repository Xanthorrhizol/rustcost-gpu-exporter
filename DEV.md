

Developer notes, build instructions, scripts



```

cd C:\code\rustcost\rustcost-gpu-exporter
docker build --build-arg APP_NAME=rustcost-gpu-exporter -t rustcost-gpu-exporter .
docker tag rustcost-gpu-exporter kimc1992/rustcost-gpu-exporter:0.0.1
docker push kimc1992/rustcost-gpu-exporter:0.0.1

```