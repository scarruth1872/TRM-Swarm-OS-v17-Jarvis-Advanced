FROM node:20-alpine
WORKDIR /app
COPY package*.json ./
RUN npm install
COPY . .
EXPOSE 5183
CMD ["npx", "vite", "--host", "0.0.0.0", "--port", "5183"]
