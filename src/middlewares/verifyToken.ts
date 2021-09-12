import jwt from 'jsonwebtoken';
import { Request, Response, NextFunction } from 'express';

export interface AuthenticatedReq extends Request {
  user?: {
    id: string | number;
  };
}

function verifyToken(req: AuthenticatedReq, res: Response, next: NextFunction) {
  const token = req.header('Authorization');
  if (!token) res.status(401).send('Access Denied');

  try {
    if (token && process.env.SECRET) {
      const obj = jwt.verify(token, process.env.SECRET);
      if (obj) req.user = <any>obj;
      next();
    }
  } catch (err) {
    res.status(400).send('invalid token');
  }
}

export default verifyToken;
