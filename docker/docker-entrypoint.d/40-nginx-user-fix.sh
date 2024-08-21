#!/usr/bin/env sh

sed "s/user  nginx;/user  root;/g" -i /etc/nginx/nginx.conf
