#!/bin/sh
set -eu
: "${NGINX_PORT:=4174}"
: "${CPM_API_UPSTREAM:=127.0.0.1:18081}"
export NGINX_PORT CPM_API_UPSTREAM
envsubst '${NGINX_PORT} ${CPM_API_UPSTREAM}' < /etc/cpm/nginx.conf.template > /tmp/nginx.conf
exec nginx -c /tmp/nginx.conf -g 'daemon off;'
