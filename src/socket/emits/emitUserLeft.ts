import { getUsersInRoom } from '../../repository/user';
import { Server } from 'socket.io';
import type { User } from '../../types';

const emitUserLeft = async (io: Server, { user }: { user: User }) => {
  if (!user.room) {
    console.error('User not in room.');
    return;
  }

  return io.in(user.room).emit('user-left', { removedUser: user, usersInRoom: await getUsersInRoom(user.room) });
};

export default emitUserLeft;
