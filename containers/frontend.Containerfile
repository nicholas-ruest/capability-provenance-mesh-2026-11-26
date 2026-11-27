FROM docker.io/library/node:24.11.1-bookworm-slim AS build
WORKDIR /src/frontend
COPY frontend/package.json frontend/package-lock.json ./
RUN npm ci
COPY frontend ./
ARG VITE_API_URL=/api
RUN npm run build

FROM docker.io/library/nginx:1.29-alpine
COPY containers/nginx.conf /etc/nginx/conf.d/default.conf
COPY --from=build /src/frontend/dist /usr/share/nginx/html
EXPOSE 8080
