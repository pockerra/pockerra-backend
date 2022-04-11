import { User } from '../types';
import { model } from 'mongoose';
import userSchema from '../schema/userSchema';

export const userModel = model<User>('User', userSchema);
