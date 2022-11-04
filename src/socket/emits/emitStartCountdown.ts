import { Socket } from 'socket.io';
import { RoomName } from '../../types';
import { getRoomByName } from '../../repository/room';

const emitStartCountdown = async (socket: Socket, { roomName }: { roomName: RoomName }) => {
  return socket.to(roomName).emit('start-countdown', { room: await getRoomByName(roomName) });
};

export default emitStartCountdown;
