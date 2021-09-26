type UserId = string
type Room = string

interface User {
    id: UserId;
    name: string;
    room?: Room;
}

export {User,UserId,Room}