import { Schema } from 'mongoose';
import { User } from '../types';

const userSchema = new Schema<User>({
  id: { type: String, required: true },
  name: { type: String, required: true },
  room: { type: String, required: true },
  card: { type: [String], required: true },
});

export default userSchema;
