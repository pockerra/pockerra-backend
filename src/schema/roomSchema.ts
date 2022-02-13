import { Schema } from 'mongoose';
import { Room } from '../types';

const roomSchema = new Schema<Room>({
  name: {
    type: String,
    required: true,
  },
  hidden: {
    type: Boolean,
    required: true,
  },
});

export default roomSchema;
