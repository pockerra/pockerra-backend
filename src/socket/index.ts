import { Socket, Server } from 'socket.io';
import { DefaultEventsMap } from 'socket.io/dist/typed-events';
import { RoomName, UserId } from '../types';
import { Card } from '../types/user';
import { addUser, getUser, removeUser, resetCards, selectCard } from '../repository/user';
import { addRoom, isRoomCreated, revealCards, startOver } from '../repository/room';
import { emitRevealed, emitStarted, emitUserJoined, emitUserLeft } from './emits';
import emitSelectedCard from './emits/emitSelectedCard';
import emitStartCountdown from './emits/emitStartCountdown';

const socketCallback = async (socket: Socket<DefaultEventsMap, DefaultEventsMap, DefaultEventsMap>, io: Server) => {
  socket.on('disconnect', async () => {
    const user = await getUser(socket.id);
    if (user) {
      await removeUser(user.id);
      await emitUserLeft(io, { user });
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
      await emitUserJoined(io, { user, name });
    }
  });

  socket.on('select-card', async (data: { card: Card; room: RoomName; userId: UserId }) => {
    await selectCard(data.userId, data.card);
    await emitSelectedCard(io, { data });
  });

  socket.on('reveal', async ({ roomName }: { roomName: RoomName }) => {
    await revealCards(roomName);
    await emitRevealed(io, { roomName });
  });

  socket.on('start-countdown', async ({ roomName }: { roomName: RoomName }) => {
    await emitStartCountdown(socket, { roomName });
  });

  socket.on('start', async ({ roomName }: { roomName: RoomName }) => {
    await startOver(roomName);
    await resetCards(roomName);
    await emitStarted(io, { roomName });
  });
};

export default socketCallback;
