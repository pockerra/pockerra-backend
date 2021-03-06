import express from 'express';
import cors from 'cors';
import dotenv from 'dotenv';
import { Server } from 'socket.io';
import http from 'http';
import socketCallback from './socket';
import { connect, connection } from 'mongoose';

const app = express();
const PORT = process.env.PORT || 3001;

dotenv.config();
// Middlewares
app.use(cors());
app.use(express.json());

const server = http.createServer(app);

app.get('/', (req, res) => {
  return res.send(`<html lang="en">
<head><title>API server</title></head><body><h1>This is API server</h1></body>
</html>`);
});

const connectToDB = async () => {
  if (connection?.readyState) return true;
  await connect(process.env.MONGODB_URI || 'mongodb://127.0.0.1:27017', { dbName: 'pockerra' }, () => {
    console.log('✅ Connected to DB');
    return Promise.resolve();
  });
};

const io = new Server(server, {
  cors: {
    origin: process.env.CLIENT_ORIGIN || `http://localhost:8080`,
    methods: ['GET', 'POST'],
  },
});

connectToDB().then(() => {
  io.on('connection', (socket) => {
    socketCallback(socket, io);
  });
});

server.listen(PORT, () => {
  console.log(`⚡️[server]: Server is running at http://localhost:${PORT}`);
});
