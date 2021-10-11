import { Socket, Server } from 'socket.io';
import { DefaultEventsMap } from 'socket.io/dist/typed-events';
import { RoomName, UserId } from './types';
import { Card } from './types/user';
import { addUser, getUser, getUsersInRoom, removeUser, resetCards, selectCard } from './repository/users';
import { addRoom, getRooms, isRoomCreated, revealCards, startOver } from './repository/room';

const socketCallback = (socket: Socket<DefaultEventsMap, DefaultEventsMap, DefaultEventsMap>, io: Server) => {
  socket.on('disconnect', () => {
    const user = getUser(socket.id);
    if (user) {
      removeUser(user.id);
      if (user.room) io.in(user.room).emit('user-left', { removedUser: user, usersInRoom: getUsersInRoom(user.room) });
    }
  });

  socket.on('join-room', async ({ roomId, name }: { roomId: string; name: string }) => {
    const { error, user } = addUser({ id: socket.id, name, room: roomId });

    if (user && !isRoomCreated(user.room)) {
      addRoom(user.room);
    }

    if (error) return console.error(error);

    if (user) {
      await socket.join(user.room);
      io.in(user.room).emit('user-joined', {
        joinedUser: name,
        usersInRoom: getUsersInRoom(user.room),
        room: getRooms(),
      });
    }
  });

  socket.on('select-card', (data: { card: Card; room: RoomName; userId: UserId }) => {
    selectCard(data.userId, data.card);
    io.in(data.room).emit('selected-card', { data, usersInRoom: getUsersInRoom(data.room) });
  });

  socket.on('reveal', ({ roomName }: { roomName: RoomName }) => {
    revealCards(roomName);
    io.in(roomName).emit('revealed', { room: getRooms() });
  });

  socket.on('start-countdown', ({ roomName }: { roomName: RoomName }) => {
    socket.to(roomName).emit('start-countdown', { room: getRooms() });
  });

  socket.on('start', ({ roomName }: { roomName: RoomName }) => {
    startOver(roomName);
    resetCards(roomName);
    io.in(roomName).emit('started', { room: getRooms(), usersInRoom: getUsersInRoom(roomName) });
  });
};

export default socketCallback;
