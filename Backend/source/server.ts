import https from 'https';
import http from 'http';
import express, { Express } from 'express';
import morgan from 'morgan';
import routes from './routes/routes';
import { initLCD } from './lib/terra/lcd';
import { initWallet } from './lib/terra/wallet';
import { initLogger, logger } from './services/logger';
import { initWorkers } from './jobs/jobs';

require('dotenv').config();
const router: Express = express();


/** Logging */
router.use(morgan('dev'));
var fs = require('fs');
var util = require('util');
var log_file = fs.createWriteStream(__dirname + '/debug.log', {flags : 'w'});
var log_stdout = process.stdout;

console.log = function(d) { //
  log_file.write(util.format(d) + '\n');
  log_stdout.write(util.format(d) + '\n');
};

initLogger();
/** Parse the request */
router.use(express.urlencoded({ extended: false }));
/** Takes care of JSON data */
router.use(express.json());

/** RULES OF OUR API */
router.use((req, res, next) => {
    res.setHeader("Access-Control-Allow-Origin", "*");
    res.setHeader("Access-Control-Allow-Credentials", "true");
    res.setHeader("Access-Control-Allow-Methods", "GET,HEAD,OPTIONS,POST,PUT");
    res.setHeader("Access-Control-Allow-Headers", "Access-Control-Allow-Headers, Origin,Accept, X-Requested-With, Content-Type, Access-Control-Request-Method, Access-Control-Request-Headers");
    // set the CORS method headers
    if (req.method === 'OPTIONS') {
        //res.header('Access-Control-Allow-Methods', 'GET PATCH DELETE POST');
        return res.status(200).json({});
    }
    next();
});

/** Routes */
router.use('/', routes);

/** Error handling */
router.use((req, res, next) => {
    const error = new Error('not found');
    return res.status(404).json({
        message: error.message
    });
});

/** Server */

if(process.env.MODE != "debug") {
    var options = {
        ch: fs.readFileSync('/ch.pem'),
        key: fs.readFileSync('/key.pem'),
        cert: fs.readFileSync('/cert.cert')
    };    
    const httpServer = https.createServer(options, router);
    const PORT: any = process.env.PORT ?? 6060;
    httpServer.listen(PORT, () => console.log(`The server is running on port ${PORT}`));
} else {
    const httpServer = http.createServer(router);
    const PORT: any = process.env.PORT ?? 6060;
    httpServer.listen(PORT, () => console.log(`The server is running on port ${PORT}`));
}
/** Terra */
const terra_init = async () => 
{
    await initLCD(process.env.TERRA_URL!, process.env.TERRA_CHAIN_ID!);
    console.log(`initWallet`);
    initWallet();
    console.log(`initWorkers`);
    await initWorkers();
    console.log(`done`);
}
terra_init();

logger.info(process.env);

