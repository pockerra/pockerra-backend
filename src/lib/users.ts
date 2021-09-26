import {Room, User, UserId} from "../types";

const users: Array<User> = [];

const addUser = ({ id, name, room }:User) => {
    name = name.trim().toLowerCase();
    room = room?.trim().toLowerCase();

    const existingUser = users.find(
        user => user.room === room && user.name === name
    );

    if (!name || !room) return { error: 'Username and room are required.' };
    if (existingUser) return { error: 'Username already exists.' };

    const user = { id, name, room };

    users.push(user);

    return { user };
};

const removeUser = (id : UserId) => {
    const index = users.findIndex(user => user.id === id);

    if (index !== -1) return users.splice(index, 1)[0];
};

const getUser = (id: UserId) => users.find(user => user.id === id);

const getUsersInRoom = (room: Room) => users.filter(user => user.room === room);

export { addUser, removeUser, getUser, getUsersInRoom };