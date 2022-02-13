// noinspection JSIgnoredPromiseFromCall

import { RoomName, User, UserId } from '../types';
import { Card } from '../types/user';
import { model, Schema } from 'mongoose';

const userSchema = new Schema<User>({
  id: { type: String, required: true },
  name: { type: String, required: true },
  room: { type: String, required: true },
  card: { type: Number, required: false },
});

const userModel = model<User>('User', userSchema);

const addUser = async ({ id, name, room }: User) => {
  name = name.trim().toLowerCase();
  room = room?.trim().toLowerCase();

  const existingUser = await userModel.exists({ id, room });

  if (!name || !room) return { error: 'Username and room are required.' };
  if (existingUser) return { error: 'Username already exists.' };

  const user = { id, name, room };

  const doc = await userModel.create(user);

  await doc.save();

  return { user };
};

const removeUser = async (id: UserId) => {
  await userModel.deleteOne({ id });

  return true;
};

const getUser = async (id: UserId) => {
  return userModel.findOne({ id });
};

const selectCard = async (id: UserId, card: Card) => {
  const user = await getUser(id);

  if (!user) return { error: 'User not found.' };
  await userModel.updateOne({ id }, { $set: { card } });

  return true;
};

const resetCards = async (roomName: RoomName) => {
  await userModel.updateMany({ room: roomName }, { $set: { card: 0 } });

  return getUsersInRoom(roomName);
};

const getUsersInRoom = async (room: RoomName) => {
  return userModel.find({ room });
};

export { addUser, removeUser, getUser, getUsersInRoom, selectCard, resetCards };
