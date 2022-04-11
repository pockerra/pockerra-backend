import { Room } from '../types';
import { model } from 'mongoose';
import roomSchema from '../schema/roomSchema';

export const roomModel = model<Room>('Room', roomSchema);
