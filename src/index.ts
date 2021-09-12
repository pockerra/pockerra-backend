import express, {response} from 'express';
import cors from 'cors';
import authRoutes from './routes/auth';
import mongoose from 'mongoose';
import dotenv from 'dotenv';
import setTimer from './routes/setTimer';
import {Server} from "socket.io";
import http from "http"


const app = express();
const PORT = process.env.PORT || 3001;

dotenv.config();
// Middlewares
app.use(cors());
app.use(express.json());

// Routes
app.use('/api/user', authRoutes);
app.use('/api/set-timer', setTimer);
const server = http.createServer(app)

mongoose.connect(
    process.env.DB_CONNECT || 'mongodb://127.0.0.1:27017',
    () => {
        console.log('✅ Connected to DB');
    }
);

app.get('/', (req, res) => {
    return res.send(`<html lang="en">
<head><title>API server</title></head><body><h1>This is API server</h1></body>
</html>`)
})


const io = new Server(server, {
    cors: {
        origin: `http://localhost:8080`,
        methods: ['GET', 'POST']
    }
})

io.on('connection', (socket) => {
    console.log('user connected: ' ,socket.id)

    socket.on('disconnect', () => {
        console.log('user disconnected: ' + socket.id)
    })
})

server.listen(PORT, () => {
    console.log(`⚡️[server]: Server is running at http://localhost:${PORT}`);
});