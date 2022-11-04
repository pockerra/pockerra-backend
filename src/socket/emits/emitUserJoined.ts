import { Server } from 'socket.io';
import { User } from '../../types';
import { getUsersInRoom } from '../../repository/user';
import { getRoomByName } from '../../repository/room';

const emitUserJoined = async (io: Server, { user, name }: { user: User; name: string }) => {
  if (!user.room) {
    console.error('User not in room.');

    return;
  }

  return io.in(user.room).emit('user-joined', {
    joinedUser: name,
    usersInRoom: await getUsersInRoom(user.room),
    room: await getRoomByName(user.room),
  });
};

export default emitUserJoined;
