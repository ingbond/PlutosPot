
import express from 'express';
const router = express.Router();

var lotteryController = require('../controllers/lottery.controller');
var playersController = require('../controllers/players.controller');
router.get('/history', playersController.getHistory);
router.get('/player/:addr', playersController.getPlayerData);
router.get('/airdrops/:addr', playersController.getAirdrops);

router.get('/charity', lotteryController.getCharityEndowements);
router.get('/lotteries', lotteryController.getLotteriesInfo);
router.get('/sum', lotteryController.checkSum);
router.get('/lotteryaddr', lotteryController.getLotteryAddr);
export = router;