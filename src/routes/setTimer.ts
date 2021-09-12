import { Router } from 'express';
import verifyToken, { AuthenticatedReq } from '../middlewares/verifyToken';
import User from '../model/User';
const router = Router();


// TODO: change this
router.post('/', verifyToken, async (req: AuthenticatedReq, res) => {
  const user = await User.findOne({ _id: req.user?.id });
  res.send(user);
});

export default router;
