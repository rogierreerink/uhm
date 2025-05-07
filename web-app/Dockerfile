# Build
FROM node:23-alpine AS builder
WORKDIR /app
COPY . .
RUN yarn install --frozen-lockfile
RUN yarn run build

# Package
FROM node:23-alpine
WORKDIR /app
COPY --from=builder /app/package.json ./package.json
COPY --from=builder /app/build ./build/
COPY --from=builder /app/node_modules ./node_modules/
COPY package.json ./
EXPOSE 3000
ENV NODE_ENV=production
CMD [ "node", "build" ]
