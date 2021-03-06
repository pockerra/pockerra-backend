// noinspection JSIgnoredPromiseFromCall
import { RoomName, User, UserId } from '../types';
import { Card } from '../types/user';
import { userModel } from '../model/user';

const addUser = async ({ id, name, room }: User) => {
  name = name.trim().toLowerCase();
  room = room?.trim().toLowerCase();
  if (!name || !room) return { error: 'Username and room are required.' };

  const userExists = await userModel.exists({ id, room });
  if (userExists) return { error: 'Username already exists.' };

  const user = { id, name, room };
  const doc = await userModel.create(user);
  try {
    await doc.save();
    return { user };
  } catch (err) {
    return { error: err };
  }
};

const removeUser = async (id: UserId) => {
  await userModel.deleteOne({ id });
};

const getUser = async (id: UserId) => {
  return userModel.findOne({ id });
};

const selectCard = async (id: UserId, card: Card) => {
  const user = await getUser(id);

  if (!user) return { error: 'User not found.' };
  await userModel.updateOne({ id }, { $set: { card } });
};

const resetCards = async (roomName: RoomName) => {
  await userModel.updateMany({ room: roomName }, { $set: { card: 0 } });

  return getUsersInRoom(roomName);
};

const getUsersInRoom = async (room: RoomName) => {
  return userModel.find({ room });
};

export { addUser, removeUser, getUser, getUsersInRoom, selectCard, resetCards };
