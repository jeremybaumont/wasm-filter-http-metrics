FROM envoyproxy/envoy:v1.19-latest
ENTRYPOINT /usr/local/bin/envoy -c /etc/envoy.yaml -l trace 
