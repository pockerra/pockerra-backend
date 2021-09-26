import {Socket} from "socket.io";
import {DefaultEventsMap} from "socket.io/dist/typed-events";

const socketCallback = (socket: Socket<DefaultEventsMap,DefaultEventsMap,DefaultEventsMap>) => {
    socket.on('disconnect', () => {
        console.log('user disconnected: ' + socket.id)
    })

    socket.on('join-room', async ({roomId, name}: { roomId: string, name: string }) => {
        await socket.join(roomId)
        socket.to(roomId).emit('user-joined', {joinedUser: name});
    })


    socket.on('select-card', (data: {card: number, room: string, user: string}) => {
        console.log(`card selected`,data)

        socket.to(data.room).emit('selected', data);
    })
}

export default socketCallback