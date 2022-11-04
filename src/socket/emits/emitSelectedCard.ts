import { Server } from 'socket.io';
import { RoomName, UserId } from '../../types';
import { Card } from '../../types/user';
import { getUsersInRoom } from '../../repository/user';

const emitSelectedCard = async (io: Server, { data }: { data: { card: Card; room: RoomName; userId: UserId } }) => {
  return io.in(data.room).emit('selected-card', { data, usersInRoom: await getUsersInRoom(data.room) });
};

export default emitSelectedCard;
