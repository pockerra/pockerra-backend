import { Socket, Server } from 'socket.io';
import { DefaultEventsMap } from 'socket.io/dist/typed-events';
import { RoomName, UserId } from './types';
import { Card } from './types/user';
import { addUser, getUser, getUsersInRoom, removeUser, resetCards, selectCard } from './repository/user';
import { addRoom, getRoomByName, getRooms, isRoomCreated, revealCards, startOver } from './repository/room';

const socketCallback = async (socket: Socket<DefaultEventsMap, DefaultEventsMap, DefaultEventsMap>, io: Server) => {
  socket.on('disconnect', async () => {
    const user = await getUser(socket.id);
    if (user) {
      await removeUser(user.id);
      if (user.room)
        io.in(user.room).emit('user-left', { removedUser: user, usersInRoom: await getUsersInRoom(user.room) });
    }
  });

  socket.on('join-room', async ({ roomId, name }: { roomId: string; name: string }) => {
    const { error, user } = await addUser({ id: socket.id, name, room: roomId });

    if (user && !(await isRoomCreated(user.room))) {
      await addRoom(user.room);
    }

    if (error) return console.error(error);

    if (user) {
      await socket.join(user.room);
      io.in(user.room).emit('user-joined', {
        joinedUser: name,
        usersInRoom: await getUsersInRoom(user.room),
        room: await getRoomByName(user.room),
      });
    }
  });

  socket.on('select-card', async (data: { card: Card; room: RoomName; userId: UserId }) => {
    await selectCard(data.userId, data.card);
    io.in(data.room).emit('selected-card', { data, usersInRoom: await getUsersInRoom(data.room) });
  });

  socket.on('reveal', async ({ roomName }: { roomName: RoomName }) => {
    await revealCards(roomName);
    io.in(roomName).emit('revealed', { room: await getRoomByName(roomName) });
  });

  socket.on('start-countdown', async ({ roomName }: { roomName: RoomName }) => {
    socket.to(roomName).emit('start-countdown', { room: await getRoomByName(roomName) });
  });

  socket.on('start', async ({ roomName }: { roomName: RoomName }) => {
    await startOver(roomName);
    await resetCards(roomName);
    io.in(roomName).emit('started', {
      room: await getRoomByName(roomName),
      usersInRoom: await getUsersInRoom(roomName),
    });
  });
};

export default socketCallback;
