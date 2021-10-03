import { Socket, Server } from 'socket.io';
import { DefaultEventsMap } from 'socket.io/dist/typed-events';
import { addUser, getUsersInRoom, removeUser, selectCard } from './lib/users';
import { Room, UserId } from './types';
import { Card } from './types/user';

const socketCallback = (socket: Socket<DefaultEventsMap, DefaultEventsMap, DefaultEventsMap>, io: Server) => {
  socket.on('disconnect', () => {
    removeUser(socket.id);
  });

  socket.on('join-room', async ({ roomId, name }: { roomId: string; name: string }) => {
    const { error, user } = addUser({ id: socket.id, name, room: roomId });
    if (error) return console.error(error);
    if (user) {
      await socket.join(user.room);
      io.in(user.room).emit('user-joined', { joinedUser: name, usersInRoom: getUsersInRoom(user.room) });
    }
  });

  socket.on('select-card', (data: { card: Card; room: Room; userId: UserId }) => {
    selectCard(data.userId, data.card);
    io.in(data.room).emit('selected-card', { data, usersInRoom: getUsersInRoom(data.room) });
  });
};

export default socketCallback;
