import { Router } from 'express';
import { loginValidation } from '../../validations';
import User from '../../model/User';
import bcrypt from 'bcryptjs';
import jwt from 'jsonwebtoken';

const LoginRouter = Router();
LoginRouter.post('/login', async (req, res) => {
  const errors = loginValidation(req.body);
  if (errors) {
    return res.status(400).send(errors);
  }

  // Check if Email currect
  const user = await User.findOne({ email: req.body.email });
  if (!user) {
    return res.status(400).send({ message: 'Email or password is wrong' });
  }

  // @ts-ignore
  const validPass = await bcrypt.compare(req.body.password, user.password);
  if (!validPass) {
    return res.status(400).send({ message: 'Email or password is wrong' });
  }

  if (process.env.SECRET) {
    const token = jwt.sign({ id: user._id }, process.env.SECRET);
    return res.header('auth-token', token).send(token);
  } else console.error('"SECRET" environment variable is not defined');

  return res.status(501).send('unknown  error');
});

export default LoginRouter;
