import { RoomName, User, UserId } from '../types';
import { Card } from '../types/user';

let users: Array<User> = [];

const addUser = ({ id, name, room }: User) => {
  name = name.trim().toLowerCase();
  room = room?.trim().toLowerCase();

  const existingUser = users.find((user) => user.room === room && user.name === name);

  if (!name || !room) return { error: 'Username and room are required.' };
  if (existingUser) return { error: 'Username already exists.' };

  const user = { id, name, room };

  users.push(user);

  return { user };
};

const removeUser = (id: UserId) => {
  const index = users.findIndex((user) => user.id === id);

  if (index !== -1) return users.splice(index, 1)[0];
};

const getUser = (id: UserId) => users.find((user) => user.id === id);

const selectCard = (id: UserId, card: Card) => {
  users = users.map((user) => {
    if (user.id === id) {
      user.card = card;
    }
    return user;
  });
};

const resetCards = (roomName: RoomName) => {
  users = users.map((user) => {
    if (user.room === roomName) {
      user.card = 0;
    }
    return user;
  });
};

const getUsersInRoom = (room: RoomName) => users.filter((user) => user.room === room);

export { addUser, removeUser, getUser, getUsersInRoom, selectCard, resetCards };
