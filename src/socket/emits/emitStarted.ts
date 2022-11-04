import { getRoomByName } from '../../repository/room';
import { getUsersInRoom } from '../../repository/user';
import { Server } from 'socket.io';

const emitStarted = async (io: Server, { roomName }: { roomName: string }) => {
  io.in(roomName).emit('started', {
    room: await getRoomByName(roomName),
    usersInRoom: await getUsersInRoom(roomName),
  });
};

export default emitStarted;
