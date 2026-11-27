FROM docker.io/library/node:24.11.1-bookworm-slim AS build
WORKDIR /src/frontend
COPY frontend/package.json frontend/package-lock.json ./
RUN npm ci
COPY frontend ./
COPY api /src/api
ARG VITE_API_URL=/api
RUN npm run build

FROM docker.io/library/nginx:1.29-alpine
COPY containers/nginx.conf /etc/cpm/nginx.conf.template
COPY --chmod=0755 containers/frontend-entrypoint.sh /usr/local/bin/cpm-frontend-entrypoint
COPY --from=build /src/frontend/dist /usr/share/nginx/html
EXPOSE 4174
ENTRYPOINT ["/usr/local/bin/cpm-frontend-entrypoint"]
