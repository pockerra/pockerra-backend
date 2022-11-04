import { Server } from 'socket.io';
import type { RoomName } from '../../types';
import { getRoomByName } from '../../repository/room';

const emitRevealed = async (io: Server, { roomName }: { roomName: RoomName }) => {
  return io.in(roomName).emit('revealed', { room: await getRoomByName(roomName) });
};

export default emitRevealed;
