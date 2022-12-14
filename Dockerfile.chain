FROM node:18-alpine3.15
RUN apk update --no-cache && apk add --no-cache curl git jq
COPY ./contract/ /src/
WORKDIR /src/
RUN yarn
ENTRYPOINT npx hardhat node --hostname 0.0.0.0
