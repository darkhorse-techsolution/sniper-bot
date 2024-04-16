import React, { useEffect, useState, useRef } from "react";
import {
  Box,
  MenuItem,
  TextField,
  Typography,
  Button,
  List,
  ListItem,
  CircularProgress,
  Grid,
  Chip,
  Slider,
  Snackbar,
  Alert,
  Paper,
  AppBar,
  Toolbar
} from "@mui/material";

import axios from "axios";
import { SERVER_URL, SOCKET_URL } from "./constants/settings";

const swapStates = [
  { text: "New", theme: "secondary" },
  { text: "Pending", theme: "primary.main" },
  { text: "Swapped", theme: "primary.dark" },
  { text: "failed", theme: "secondary" },
];

const QuickSwapBoardData = {
  HEX: [
    { symbol: "10K", value: 10000 },
    { symbol: "50K", value: 50000 },
    { symbol: "100K", value: 100000 },
    { symbol: "200K", value: 200000 },
    { symbol: "500K", value: 500000 },
    { symbol: "1M", value: 1000000 },
  ],
  PLS: [
    { symbol: "1M", value: 1000000 },
    { symbol: "2M", value: 2000000 },
    { symbol: "5M", value: 5000000 },
    { symbol: "10M", value: 10000000 },
    { symbol: "20M", value: 20000000 },
    { symbol: "50M", value: 50000000 },
  ],
};

const SimpleSnackbar = ({ message, vertical, horizontal, open, handleSnackbar, serverity = "success" }) => {
  return (
    <Snackbar
      anchorOrigin={{ vertical, horizontal }}
      autoHideDuration={6000}
      open={open}
      onClose={() => handleSnackbar(false)}
      key={vertical + horizontal}
    >
      <Alert
        // onClose={() => handleSnackbar(false)}
        severity={serverity}
        variant="filled"
        sx={{ width: '100%' }}
      >
        {message}
      </Alert>
    </Snackbar>
  );
}

const DirectSwapBoard = ({
  selectedToken,
  tokens,
  handleTokenChange,
  handleTargetToken,
  handleSubmit,

  priorityFee,
  setPriorityFee,
  slippageTolerance,
  setSlippageTolerance,
  quickSwapValue,
  setQuickSwapValue,
  setWillDoQuickSwap,
  willDoQuickSwap,
  customAmountDirectSwap,
  setCustomAmountDirectSwap,
  swapPendingStates,
  setSwapPendingStates
}) => {
  const [currentToken, setCurrentToken] = useState({});

  const handleSliderChange = (event, newValue) => {
    setSlippageTolerance(newValue);
  };
  // const handleCurrentToken = (event) => {
  //   tokens.map((item, key) => {
  //     if(item.symbol === event.target.value) setCurrentToken(item.address);
  //   })
  // }
  useEffect(() => {
    setCurrentToken(selectedToken);
  }, [selectedToken]);

  return (
    <form onSubmit={handleSubmit} style={{ flex: 1 }}>
      <Box
        sx={{
          display: "flex",
          flexDirection: "column",
          gap: 3,
          padding: 3,
          backgroundColor: "#f9f9f9",
          borderRadius: 2,
          boxShadow: "0 2px 8px rgba(0, 0, 0, 0.1)",
          flex: 1,
          position: "sticky",
          top: 0,
          zIndex: 10,
        }}
      >
        <Typography sx={{ fontWeight: "bold", fontSize: "1.5rem" }}>
          Direct Swap
        </Typography>
        <TextField
          label="Target Token Address"
          name="token"
          fullWidth
          onChange={handleTargetToken}
          helperText="Paste your token address here."
          required
        ></TextField>
        <TextField
          select
          label="Payment Token"
          name="token"
          fullWidth
          onChange={(Event) => {
            handleTokenChange(Event);
            // handleCurrentToken(Event);
          }}
          // helperText={currentToken}
          helperText={
            currentToken.address
              ? currentToken.address
              : "Choose the token you want to purchase."
          }
          required
        >
          {tokens.map((token) => (
            <MenuItem key={token.address} value={token.address}>
              {token.symbol}
            </MenuItem>
          ))}
        </TextField>

        <TextField
          label="Priority Fee (Tip) in GWEI"
          type="number"
          variant="outlined"
          value={priorityFee}
          onChange={(e) => setPriorityFee(Number(e.target.value))}
        />
        <Box sx={{ padding: 0 }}>
          <Typography
            sx={{
              fontWeight: "normal",
              fontSize: "1rem",
              padding: 0,
              color: "GrayText",
            }}
          >
            Slippage Tolerance: {slippageTolerance / 10}%
          </Typography>
          <Slider
            aria-label="Slippage Tolerance"
            value={slippageTolerance}
            onChange={handleSliderChange}
            min={0}
            max={500}
          />
        </Box>
        <Box
          sx={{
            display: "flex",
            flexDirection: { xs: "column", md: "row" },
            flexWrap: "wrap",
            gap: 1,
            flex: 1,
          }}
        >
          {selectedToken["symbol"] &&
            QuickSwapBoardData[selectedToken["symbol"]].map((item, key) => {
              let { value, symbol } = item;

              return (
                <Box key={key} sx={{ flex: { xs: "1 1 100%", md: "1 1 30%" } }}>
                  <Button
                    type="button"
                    variant="outlined"
                    color="primary"
                    size="large"
                    onClick={() => {
                      setQuickSwapValue(value);
                      setWillDoQuickSwap(willDoQuickSwap + 1);
                      setCustomAmountDirectSwap(0);
                      setSwapPendingStates([
                        ...swapPendingStates,
                        {
                          value,
                          status: "0x2",
                          startedAt: new Date().toISOString(),
                        },
                      ])
                    }}
                    sx={{
                      backgroundColor:
                        value === quickSwapValue ? "#1976d2" : "transparent",
                    }}
                    fullWidth
                  >
                    <Typography
                      fontSize="large"
                      color={value === quickSwapValue ? "white" : "secondary"}
                    >
                      {symbol}
                    </Typography>
                    &nbsp;
                    <Typography
                      color={value === quickSwapValue ? "yellow" : "primary"}
                    >
                      {" "}
                      {selectedToken.symbol}
                    </Typography>
                  </Button>
                </Box>
              )
            })}
        </Box>
        <Box
          sx={{
            display: "flex",
            flexDirection: { xs: "column", md: "row", md: "column" },
            flexWrap: "wrap",
            gap: 1,
            flex: 1,
          }}
        >
          <TextField
            label="Custom Value"
            name="tk_input"
            defaultValue=""
            // value={currentSelectedElem !== "tk_input" ? 0 : selectedTag }
            size="small"
            // sx={{ width: "70%" }}
            value={customAmountDirectSwap}
            onChange={(event) => {
              setCustomAmountDirectSwap(event.target.value);
              setQuickSwapValue(0);
            }}
          ></TextField>
          <Button
            variant="contained"
            color="primary"
            onClick={() => {
              setWillDoQuickSwap(willDoQuickSwap + 1);
              setSwapPendingStates([
                ...swapPendingStates,
                {
                  value: parseInt(customAmountDirectSwap),
                  status: "0x2",
                  startedAt: new Date().toISOString()
                }
              ])
              // setStatus(1);
            }}
          >Swap</Button>
        </Box>
        {/* <Button type="submit" variant="contained" color="primary">
          Select
        </Button> */}
        <TransactionList txStatusList={swapPendingStates} tokenSymbol={selectedToken["symbol"]} />
        <Box padding={2}></Box>
      </Box>
    </form>
  );
};

const SnipeSettingBoard = ({
  selectedToken,
  tokens,
  handleTokenChange,
  handleTargetToken,
  handleSubmit,

  priorityFee,
  setPriorityFee,
  slippageTolerance,
  setSlippageTolerance,
  quickSwapValueSnipeSetting,
  setQuickSwapValueSnipeSetting,
  customAmountSnipeSetting,
  setCustomAmountSnipeSetting
}) => {
  const [currentToken, setCurrentToken] = useState({});
  const handleSliderChange = (event, newValue) => {
    setSlippageTolerance(newValue);
  };
  // const handleCurrentToken = (event) => {
  //   tokens.map((item, key) => {
  //     if(item.symbol === event.target.value) setCurrentToken(item.address);
  //   })
  // }
  useEffect(() => {
    setCurrentToken(selectedToken);
  }, [selectedToken]);

  return (
    <form onSubmit={handleSubmit} style={{ flex: 1 }}>
      <Box
        sx={{
          display: "flex",
          flexDirection: "column",
          gap: 3,
          padding: 3,
          backgroundColor: "#f9f9f9",
          borderRadius: 2,
          boxShadow: "0 2px 8px rgba(0, 0, 0, 0.1)",
          flex: 1,
          position: "sticky",
          top: 0,
          zIndex: 10,
        }}
      >
        <Typography sx={{ fontWeight: "bold", fontSize: "1.5rem" }}>
          Snipe Interface
        </Typography>
        <TextField
          label="Target Token Address"
          name="token"
          fullWidth
          onChange={handleTargetToken}
          helperText="Paste your token address here."
          required
        ></TextField>
        <TextField
          select
          label="Payment Token"
          name="token"
          fullWidth
          onChange={(Event) => {
            handleTokenChange(Event);
            // handleCurrentToken(Event);
          }}
          // helperText={currentToken}
          helperText={
            currentToken.address
              ? currentToken.address
              : "Choose the token you want to purchase."
          }
          required
        >
          {tokens.map((token) => (
            <MenuItem key={token.address} value={token.address}>
              {token.symbol}
            </MenuItem>
          ))}
        </TextField>

        <TextField
          label="Priority Fee (Tip) in GWEI"
          type="number"
          variant="outlined"
          value={priorityFee}
          onChange={(e) => setPriorityFee(Number(e.target.value))}
        />
        <Box sx={{ padding: 0 }}>
          <Typography
            sx={{
              fontWeight: "normal",
              fontSize: "1rem",
              padding: 0,
              color: "GrayText",
            }}
          >
            Slippage Tolerance: {slippageTolerance / 10}%
          </Typography>
          <Slider
            aria-label="Slippage Tolerance"
            value={slippageTolerance}
            onChange={handleSliderChange}
            min={0}
            max={500}
          />
        </Box>
        <Box
          sx={{
            display: "flex",
            flexDirection: { xs: "column", md: "row" },
            flexWrap: "wrap",
            gap: 1,
            flex: 1,
          }}
        >
          {selectedToken["symbol"] &&
            QuickSwapBoardData[selectedToken["symbol"]].map((item, key) => {
              let { value, symbol } = item;

              return (
                <Box key={key} sx={{ flex: { xs: "1 1 100%", md: "1 1 30%" } }}>
                  <Button
                    type="button"
                    variant="outlined"
                    color="primary"
                    size="large"
                    onClick={() => {
                      setQuickSwapValueSnipeSetting(value)
                      setCustomAmountSnipeSetting(0)
                    }}
                    sx={{
                      backgroundColor:
                        value === quickSwapValueSnipeSetting ? "#1976d2" : "transparent",
                    }}
                    fullWidth
                  >
                    <Typography
                      fontSize="large"
                      color={value === quickSwapValueSnipeSetting ? "white" : "secondary"}
                    >
                      {symbol}
                    </Typography>
                    &nbsp;
                    <Typography
                      color={value === quickSwapValueSnipeSetting ? "yellow" : "primary"}
                    >
                      {" "}
                      {selectedToken.symbol}
                    </Typography>
                  </Button>
                </Box>
              )
            })}
        </Box>

        <Box
          sx={{
            display: "flex",
            flexDirection: { xs: "column", md: "row", md: "column" },
            flexWrap: "wrap",
            gap: 1,
            flex: 1,
          }}
        >
          <TextField
            label="Custom Value"
            name="tk_input"
            defaultValue=""
            // value={currentSelectedElem !== "tk_input" ? 0 : selectedTag }
            size="small"
            // sx={{ width: "70%" }}
            value={customAmountSnipeSetting}
            onChange={(event) => {
              setCustomAmountSnipeSetting(event.target.value);
              setQuickSwapValueSnipeSetting(0);
            }}
          ></TextField>
        </Box>
        <Box padding={2}></Box>
      </Box>
    </form>
  );
};

function setupWebSocket(webhookUrl) {
  return async function sendMessage(message) {
    const data = { content: message };
    const headers = { "Content-Type": "application/json" };

    try {
      const response = await fetch(webhookUrl, {
        method: "POST",
        headers: headers,
        body: JSON.stringify(data),
      });
    } catch (error) {
      console.error(`Request failed: ${error}`);
    }
  };
}

const SnipeItem = ({ snipeData, selectedToken, swapState, totalData, setTotalData, snipe, isRetrying }) => {
  const { address, token_0, token_1, launch_time, version } = snipeData;
  const [selectedTag, setSelectedTag] = useState("");
  const [currentSelectedElem, setcurrentSelectedElem] = useState("");
  const [quickSwapValue, setQuickSwapValue] = useState(0);

  useEffect(() => {
    if (currentSelectedElem === "tk_chip") {
      snipe(address, token_0, token_1, version);
      // setStatus(1);
    }
  }, [currentSelectedElem, quickSwapValue]);

  useEffect(() => {
    console.log("status => ", swapState);
  }, [swapState.status]);

  const handleSubmit = async (event) => {
    event.preventDefault();
    const formData = new FormData(event.currentTarget);
    const token = formData.get("token");

    console.log("Submitted Data:", {
      poolAddress: address,
      currency: selectedToken?.address,
      firstToken: token === token_0 ? token_1 : token_0,
      tokenToBuy: token,
      buyAmount: selectedTag.toString(),
    });

    await axios.post(`${SERVER_URL}/bot/pool_snipe`, {
      poolAddress: address,
      currency: selectedToken?.address,
      firstToken: token == token_0 ? token_1 : token_0,
      tokenToBuy: token,
      buyAmount: selectedTag.toString(),
    });
  };

  return (
    <form onSubmit={handleSubmit}>
      <ListItem
        sx={{
          display: "flex",
          flexDirection: "column",
          gap: 2,
          padding: 2,
          backgroundColor: "#f0f0f0",
          borderRadius: 2,
          marginBottom: 2,
        }}
      >
        <Box sx={{ position: "absolute", right: 12, top: 6, padding: "0px 4px", borderRadius: "12px 12px", bgcolor: (swapState && swapState.status) ? swapStates[swapState.status].theme : "secondary" + ".main" }} color={(swapState && swapState.status) ? swapStates[swapState.status].theme : "secondary"} >
          <Typography color="white" fontSize="small">{(swapState && swapState.status) ? swapStates[swapState.status].text : "New"}</Typography>
        </Box>
        <Typography color="white" fontSize="small">{(isRetrying && isRetrying.status) ? "retrying..." + isRetrying.retryCounts + " times" : ""}</Typography>
        {/* Responsive Address and Launch Time */}
        <Grid container spacing={2} alignItems="center">
          <Grid item xs={12} md={6}>
            <Typography variant="body1" sx={{ wordBreak: "break-word" }}>
              <strong>Address:</strong> {address}
            </Typography>
          </Grid>
          <Grid item xs={12} md={6}>
            <Typography variant="body1">
              <strong>Launch Time:</strong> {launch_time}
            </Typography>
          </Grid>
        </Grid>

        {/* Input bar and Snipe button */}
        <Box sx={{ display: "flex", width: "100%", flexDirection: "column" }}>
          <Typography color="secondary" fontSize="large" sx={{}}>
            Available Tokens
          </Typography>
          <Typography sx={{ color: "#292929" }} >
            {token_0}
          </Typography>
          <Typography sx={{ color: "#292929" }} >
            {token_1}
          </Typography>
        </Box>
      </ListItem>
    </form>
  );
};

const TransactionList = ({
  txStatusList: transactions,
  tokenSymbol
}) => {
  // const [transactions, setTransactions] = useState([]);

  // Simulate fetching transactions
  useEffect(() => {
    const fetchTransactions = async () => {
      // Replace this with your real data fetching logic
      // const txStatusList = [
      //     {
      //         id: 'tx1',
      //         amount: '10 ETH',
      //         submittedAt: new Date().toISOString(),
      //         confirmedAt: new Date(Date.now() + 2000).toISOString(),
      //         blockNumber: 1234567,
      //     },
      //     {
      //         id: 'tx2',
      //         amount: '5 ETH',
      //         submittedAt: new Date(Date.now() - 5000).toISOString(),
      //         failedAt: new Date(Date.now() - 1000).toISOString(),
      //     },
      // ];
      // setTransactions(txStatusList);
    };

    fetchTransactions();
  }, []);

  return (
    <Box sx={{ padding: 3 }}>
      <Typography variant="h4" gutterBottom>
        Queued Transactions
      </Typography>
      <List>
        {transactions.map((tx) => (
          <ListItem key={tx.id} disableGutters>
            <Paper elevation={3} sx={{ padding: 2, width: '100%', borderRadius: 2 }}>
              {/* <Typography variant="body1">
                              <strong>ID:</strong> {tx.id}
                          </Typography> */}
              <Typography variant="body1">
                <strong>Amount:</strong> {tx.value + ' ' + tokenSymbol}
              </Typography>
              <Typography variant="body1">
                <strong>Submitted At:</strong> {new Date(tx.startedAt).toLocaleString()}
              </Typography>
              {tx.confirmedAt && (
                <Typography variant="body1">
                  <strong>Confirmed At:</strong> {new Date(tx.confirmedAt).toLocaleString()}
                </Typography>
              )}
              {tx.blockNumber && (
                <Typography variant="body1">
                  <strong>Block Number:</strong> {parseInt(tx.blockNumber, 16)}
                </Typography>
              )}
              {tx.status && (
                <Typography variant="body1"
                  color={tx.status == "0x1"
                    ? "success"
                    : tx.status == "0x2"
                      ? "info"
                      : "error"
                  }>
                  <strong>Status:</strong> {
                    tx.status == "0x1"
                      ? "Success!"
                      : tx.status == "0x2"
                        ? "Pending..."
                        : "Failed"
                  }
                </Typography>
              )}
            </Paper>
          </ListItem>
        ))}
      </List>
    </Box>
  );
};

const SnipingBoard = ({
  snipingList,
  startSnipe,
  snipe,
  selectedToken,
  isSniping,
  settings,
  setQuickSwapValue,
  totalData,
  setTotalData,
  swapState,
  isRetrying
}) => (
  <Box
    sx={{
      display: "flex",
      flexDirection: "column",
      gap: 3,
      padding: 3,
      backgroundColor: "#f9f9f9",
      borderRadius: 2,
      boxShadow: "0 2px 8px rgba(0, 0, 0, 0.1)",
    }}
  >
    <Box
      sx={{
        display: "flex",
        justifyContent: "space-between",
        alignItems: "center",
        position: "sticky",
        top: 0,
        zIndex: 10,
        backgroundColor: "#f9f9f9",
        padding: 2,
        borderRadius: 2,
        boxShadow: "0 2px 8px rgba(0, 0, 0, 0.1)",
      }}
    >
      <Typography sx={{ fontWeight: "bold", fontSize: "1.5rem" }}>
        Sniping Board
      </Typography>
      <Button
        type="button"
        variant="contained"
        color="secondary"
        size="small"
        onClick={startSnipe}
        startIcon={isSniping && <CircularProgress size={20} color="inherit" />}
      >
        {isSniping ? "Stop Snipe" : "Start Snipe"}
      </Button>
    </Box>

    {snipingList.length > 0 ? (
      <Grid container spacing={2}>
        {snipingList.map((item, index) => (
          <Grid
            item
            xs={12} // Full width on extra-small screens
            sm={12} // Full width on small screens
            md={6} // Half width on medium and larger screens
            key={index}
          >
            <SnipeItem
              snipeData={item}
              selectedToken={selectedToken}
              settings={settings}
              snipe={snipe}
              setQuickSwapValue={setQuickSwapValue}
              totalData={totalData}
              setTotalData={setTotalData}
              swapState={(swapState && swapState[item.address]) ? swapState[item.address] : {}}
              isRetrying={(isRetrying && isRetrying[item.address]) ? isRetrying[item.address] : {}}
            />
          </Grid>
        ))}
      </Grid>
    ) : (
      <Typography sx={{ textAlign: "center" }}>
        No tokens added yet. Messages will appear here.
      </Typography>
    )}
  </Box>
);

const TokenSelectorForm = () => {
  const [tokensDirectSwap, setTokensDirectSwap] = useState([]);
  const [tokensSnipeSetting, setTokensSnipeSetting] = useState([]);

  const [settings, setSettings] = useState({});
  const [selectedTokenDirectSwap, setSelectedTokenDirectSwap] = useState({});
  const [selectedTokenSnipeSetting, setSelectedTokenSnipeSetting] = useState({});

  const [targetTokenDirectSwap, setTargetTokenDirectSwap] = useState("");
  const [targetTokenSnipeSetting, setTargetTokenSnipeSetting] = useState("");

  const [snipingList, setSnipingList] = useState([]);
  const [isSniping, setIsSniping] = useState(false);
  const [socket, setSocket] = useState(null);

  const [priorityFeeDirectSwap, setPriorityFeeDirectSwap] = useState(0);
  const [priorityFeeSnipeSetting, setPriorityFeeSnipeSetting] = useState(0);

  const [slippageToleranceDirectSwap, setSlippageToleranceDirectSwap] = useState(0);
  const [slippageToleranceSnipeSetting, setSlippageToleranceSnipeSetting] = useState(0);

  const [quickSwapValueDirectSwap, setQuickSwapValueDirectSwap] = useState("");
  const [quickSwapValueSnipeSetting, setQuickSwapValueSnipeSetting] = useState("");

  const [willDoQuickSwap, setWillDoQuickSwap] = useState(0);

  const [customAmountDirectSwap, setCustomAmountDirectSwap] = useState(0);
  const [customAmountSnipeSetting, setCustomAmountSnipeSetting] = useState(0);

  const [swapState, setSwapState] = useState({});
  const [totalDataSnipeSetting, setTotalDataSnipeSetting] = useState({
    selectedToken: {},
    targetToken: targetTokenSnipeSetting,
    quickSwapValue: "",
    priorityFee: priorityFeeSnipeSetting,
    slippageTolerance: slippageToleranceSnipeSetting / 10,
  });
  const [totalDataDirectSwap, setTotalDataDirectSwap] = useState({
    selectedToken: {},
    targetToken: targetTokenDirectSwap,
    quickSwapValue: "",
    priorityFee: priorityFeeDirectSwap,
    slippageTolerance: slippageToleranceDirectSwap / 10,
  });

  const [swapPendingStates, setSwapPendingStates] = useState([]);
  const [swapFinishedCount, setSwapFinishedCount] = useState(0);
  const [currentSwapReceipt, setCurrentSwapReceipt] = useState(null);
  const [isRetrying, setIsRetrying] = useState({});

  const [walletAddress, setWalletAddress] = useState("0x");
  const [wsStatus, setWsStatus] = useState(0);

  useEffect(() => {
    console.log("will do quick swap => ", willDoQuickSwap);

    if ((quickSwapValueDirectSwap > 0 && willDoQuickSwap) || (customAmountDirectSwap > 0 && willDoQuickSwap)) {
      snipe(totalDataDirectSwap.targetToken, totalDataDirectSwap.selectedToken.address, 2);
    }
  }, [willDoQuickSwap])

  const [snackbar, setSnackbar] = useState({
    open: false,
    vertical: '',
    horizontal: '',
    message: '',
    theme: '',
  });

  const fetchData = async () => {
    try {
      // const response = await axios.get(`${SERVER_URL}/settings/get_tokens`);
      // setTokens((response.data.value && response.data.value.target_tokens) || []);
      setTokensDirectSwap([
        {
          address: "0x2b591e99afE9f32eAA6214f7B7629768c40Eeb39",
          total_supply: "0x1",
          decimals: "0x12",
          symbol: "HEX",
        },
        {
          address: "0xa1077a294dde1b09bb078844df40758a5d0f9a27",
          total_supply: "0x1",
          decimals: "0x12",
          symbol: "PLS",
        },
      ]);
      setTokensSnipeSetting([
        {
          address: "0x2b591e99afE9f32eAA6214f7B7629768c40Eeb39",
          total_supply: "0x1",
          decimals: "0x12",
          symbol: "HEX",
        },
        {
          address: "0xa1077a294dde1b09bb078844df40758a5d0f9a27",
          total_supply: "0x1",
          decimals: "0x12",
          symbol: "PLS",
        },
      ]);
      // setSettings(response.data.value);
      setSettings({
        message: "Tokens received",
        value: {
          target_tokens: [
            {
              address: "0x95b303987a60c71504d99aa1b13b4da07b0790ab",
              total_supply: "0x1",
              decimals: "0x12",
              symbol: "HEX",
            },
            {
              address: "0xa1077a294dde1b09bb078844df40758a5d0f9a27",
              total_supply: "0x1",
              decimals: "0x12",
              symbol: "PLS",
            },
          ],
          slippage_tolerance: 0.5,
          gas_price: "0x4c4b40",
          token_in: [
            {
              address: "0xa1077a294dde1b09bb078844df40758a5d0f9a27",
              total_supply: "0x1",
              decimals: "0x12",
              symbol: "PLS",
            },
          ],
          buy_amount_tags: [
            "0x1",
            "0xa",
            "0x32",
            "0x64",
            "0x1f4",
            "0x3e8",
            "0x1388",
          ],
        },
      });
      // setSnipingList([
      //   {
      //     address: "0x3678d1a77a78c068d4358f7a503f6d69ee5241f4",
      //     launch_time: '"2024-12-13 21:04:56.301"',
      //     token_0: "0x2db160ceea0f1b4ca0159fcf67fccf8e549191ff",
      //     token_1: "0xa1077a294dde1b09bb078844df40758a5d0f9a27",
      //     version: 2,
      //     weth_liquidity: "0",
      //   },
      // ]);
    } catch (error) {
      console.error("Error fetching data:", error);
    }
  };

  const connectSocket = () => {
    let ws = new WebSocket(`${SOCKET_URL}/ws`);
    setSocket(ws);

    return ws;
  }

  const handleSnackbar = (value) => {
    setSnackbar({ ...snackbar, open: value })
  }

  const handleTokenChangeDirectSwap = (event) => {
    const token = tokensDirectSwap.find((t) => t.address === event.target.value);
    setSelectedTokenDirectSwap(token || {});
  };

  const handleTokenChangeSnipeSetting = (event) => {
    const token = tokensSnipeSetting.find((t) => t.address === event.target.value);
    setSelectedTokenSnipeSetting(token || {});
  };

  const handleTargetTokenDirectSwap = (event) => {
    setTargetTokenDirectSwap(event.target.value);
  };
  const handleTargetTokenSnipeSetting = (event) => {
    setTargetTokenSnipeSetting(event.target.value);
  };

  const handleSubmitDirectSwap = (event) => {
    event.preventDefault();
    const formData = new FormData(event.currentTarget);
    const token = tokensDirectSwap.find((t) => t.address === formData.get("token"));
    if (!token) {
      alert("Please select a token first!");
      return;
    }
    setSelectedTokenDirectSwap(token);
  };

  const handleSubmitSnipeSetting = (event) => {
    event.preventDefault();
    const formData = new FormData(event.currentTarget);
    const token = tokensSnipeSetting.find((t) => t.address === formData.get("token"));
    if (!token) {
      alert("Please select a token first!");
      return;
    }
    setSelectedTokenSnipeSetting(token);
  };

  const snipe = async (token0, token1, version) => {
    let swapAmount = quickSwapValueDirectSwap || customAmountDirectSwap.toString();
    swapAmount = parseInt(swapAmount, 10).toString(16);
    console.log(quickSwapValueDirectSwap || customAmountDirectSwap, quickSwapValueDirectSwap, customAmountDirectSwap, swapAmount);

    let payload = {
      "msg_type": "swap",
      "content": JSON.stringify({
        token0,
        token1,
        version,
        "settings": JSON.stringify({
          "target_token": totalDataDirectSwap.targetToken,
          "slippage_tolerance": totalDataDirectSwap.slippageTolerance,
          "amount_to_buy": swapAmount,
          "currency_token": totalDataDirectSwap.selectedToken.address,
          "gas_price": totalDataDirectSwap.priorityFee.toString(16)
        })
      })
    }
    if (socket) {
      console.log("payload => ", payload);

      socket.send(JSON.stringify(payload));
    } else {
      let ws = connectSocket();
      ws.send(payload);
    }
  }

  const startSnipe = async () => {
    let payload = {
      "msg_type": "Start sniping",
      "content": JSON.stringify({
        "target_token": totalDataSnipeSetting.targetToken,
        "slippage_tolerance": totalDataSnipeSetting.slippageTolerance,
        "amount_to_buy": totalDataSnipeSetting.quickSwapValue.toString(16),
        "currency_token": totalDataSnipeSetting.selectedToken.address,
        "gas_price": totalDataSnipeSetting.priorityFee.toString(16)
      })
    };
    payload = JSON.stringify(payload);
    console.log(payload);

    try {
      console.log("isSniping... => ", isSniping);

      if (isSniping == true) {
        let payload = {
          "msg_type": "close",
          "content": ""
        };
        socket.send(JSON.stringify(payload)); socket.close(); setIsSniping(false);
      }
      else {
        console.log(socket);

        if (socket && socket.readyState != 1) {
          connectSocket()
        }
        socket.send(payload);
      }
      // await axios.get(`${SERVER_URL}/bot/start_snipe_pool`);
    } catch (error) {
      connectSocket();

      if (isSniping == true) { socket.close(); setIsSniping(false); }
      else socket.send(payload);

      console.error("Error during sniping start", error);
    }
  };

  useEffect(() => {
    let swapData = quickSwapValueSnipeSetting || customAmountSnipeSetting.toString();
    swapData = parseInt(swapData, 10).toString(16);

    setTotalDataSnipeSetting({
      selectedToken: selectedTokenSnipeSetting,
      targetToken: targetTokenSnipeSetting,
      quickSwapValue: swapData,
      priorityFee: priorityFeeSnipeSetting,
      slippageTolerance: slippageToleranceSnipeSetting / 10,
    });
  }, [
    selectedTokenSnipeSetting,
    targetTokenSnipeSetting,
    quickSwapValueSnipeSetting,
    priorityFeeSnipeSetting,
    slippageToleranceSnipeSetting,
    customAmountSnipeSetting
  ]);

  useEffect(() => {
    setTotalDataDirectSwap({
      selectedToken: selectedTokenDirectSwap,
      targetToken: targetTokenDirectSwap,
      quickSwapValue: quickSwapValueDirectSwap,
      priorityFee: priorityFeeDirectSwap,
      slippageTolerance: slippageToleranceDirectSwap / 10,
    });
  }, [
    selectedTokenDirectSwap,
    targetTokenDirectSwap,
    quickSwapValueDirectSwap,
    priorityFeeDirectSwap,
    slippageToleranceDirectSwap,
  ]);

  useEffect(() => {
    fetchData();
    connectSocket();
  }, []);

  useEffect(() => {
    if (socket) {
      console.log(socket.readyState);
      
      // socket.send(payload);
      socket.onmessage = (event) => {
        const newMessage = JSON.parse(event.data);
        console.log("swapping", newMessage);
        if (newMessage) {
          console.log(newMessage.msg_type);

        }
        if (newMessage && newMessage.content && newMessage.content == "pong") {
          setIsSniping(true);
        }

        if (
          newMessage &&
          newMessage.msg_type &&
          newMessage.msg_type == "get wallet address"
        ) {
          let content = newMessage.content;
          setWalletAddress(content.wallet_address);
        }

        if (
          newMessage &&
          newMessage.msg_type &&
          newMessage.msg_type == "handshake"
        ) {
          let content = newMessage.content;
          console.log("content => ", content);
          
          setWsStatus(content.status);
        }

        if (
          newMessage &&
          newMessage.type &&
          newMessage.type == "start getting pool"
        ) {
          setIsSniping(true);
        }

        if (
          newMessage &&
          newMessage.msg_type &&
          newMessage.msg_type == "swapping"
        ) {
          let swapData = newMessage.content;
          console.log("swapping... ", swapData);
          setSwapState({
            ...swapState,
            [swapData.pair_address]: {
              status: 1,
              amountIn: swapData.amount_in,
              amountOut: swapData.amount_out
            }
          })
          setCurrentSwapReceipt({});
          setSnackbar({
            ...snackbar,
            open: true,
            message: `
              swapping...
              amount in: ${swapData.amount_in}
              amount out: ${swapData.amount_out}
            `
          })
        }

        if (
          newMessage &&
          newMessage.msg_type &&
          newMessage.msg_type == "swapped"
        ) {
          const swapData = newMessage.content;
          const receipt = swapData.receipt;

          console.log("swapData => ", swapData);
          setSwapState({
            ...swapState,
            [swapData.pair_address]: {
              status: receipt.status == "0x0" ? 3 : 2,
              receipt: receipt
            }
          })

          setCurrentSwapReceipt(receipt);

          if (receipt.status == "0x0") {
            setSnackbar({
              ...snackbar,
              open: true,
              message: `
                failed !!!
              `
            })
          } else {
            setSnackbar({
              ...snackbar,
              open: true,
              message: `
                Success !!!
              `
            })
          }
        }

        if (
          newMessage &&
          newMessage.msg_type &&
          newMessage.msg_type == "retrying"
        ) {
          const snipeData = newMessage.content;

          if (isSniping) {
            let retryCounts = (isRetrying && isRetrying[snipeData.pair_address]) ? isRetrying[snipeData.pair_address] : 0;
            setIsRetrying({
              ...isRetrying,
              [snipeData.pair_address]: {
                status: true,
                retryCounts: retryCounts + 1
              }
            });
          }
        }

        if (newMessage && newMessage.length) {
          setSnipingList([...newMessage]);
        }
      };

      return () => socket.close();
    }
  }, [socket]);

  useEffect(() => {
    const interval = setInterval(() => {
      if (socket && socket.send) {
        let payload = {
          "msg_type": "handshake",
          "content": ""
        }
        try {
          socket.send(JSON.stringify(payload));
          if (socket.readyState == 3) {
            setWsStatus(false);
          }
          console.log("sent!!!", socket.readyState);
        } catch (error) {
          console.log("error!!!");
          setWsStatus(false);
        } 
      } else {
        setWsStatus(false);
      }
    }, 2000);
    return () => clearInterval(interval);
  }, [socket])

  useEffect(() => {
    console.log("swap state => ", swapState);
  }, [swapState])

  useEffect(() => {
    console.log(currentSwapReceipt);
    let _swapPendingStates = swapPendingStates;
    console.log("swap pending states => ", _swapPendingStates, swapPendingStates, swapFinishedCount);

    if (_swapPendingStates && _swapPendingStates.length > 0) {
      _swapPendingStates[swapFinishedCount] = {
        value: _swapPendingStates[swapFinishedCount]?.value,
        ..._swapPendingStates[swapFinishedCount],
        ...currentSwapReceipt
      }

      if (currentSwapReceipt && _swapPendingStates && _swapPendingStates[swapFinishedCount].status != '0x2') {
        _swapPendingStates[swapFinishedCount] = {
          ..._swapPendingStates[swapFinishedCount],
          confirmedAt: new Date().toISOString()
        }
      } else {
        _swapPendingStates[swapFinishedCount] = {
          ..._swapPendingStates[swapFinishedCount],
          startedAt: new Date().toISOString()
        }
      }
    }
    setSwapPendingStates(_swapPendingStates);
    if (
      currentSwapReceipt &&
      (currentSwapReceipt.status == "0x0" || currentSwapReceipt.status == "0x1")
    )
      setSwapFinishedCount(swapFinishedCount + 1);
  }, [currentSwapReceipt])

  useEffect(() => {
    console.log("swap pending status", swapPendingStates);
  }, [swapPendingStates])

  return (
    <Box sx={{ display: "flex", flexDirection: "column", gap: 4, padding: 3 }}>
      <SimpleSnackbar
        message={snackbar.message}
        vertical={snackbar.vertical}
        horizontal={snackbar.horizontal}
        open={snackbar.open}
        handleSnackbar={handleSnackbar}
      />
      <AppBar
      position="fixed"
      sx={{
        backgroundColor: "#ffffff", // White background
        boxShadow: "none",         // Remove default shadow
        borderBottom: "1px solid #e0e0e0", // Optional border for separation
      }}
    >
      <Toolbar>
        <Box sx={{ flexGrow: 1, display: "flex", alignItems: "center" }}>
          <Typography variant="h6" sx={{ color: "#000", mr: 2 }}>
            Wallet:
          </Typography>
          <Typography variant="body1" noWrap sx={{ color: "#555" }}>
            {walletAddress || "Not Connected"}
          </Typography>
        </Box>
        <Chip
          label={`WebSocket: ${wsStatus ? "Connected" : "Disconnected"}`}
          sx={{
            backgroundColor: wsStatus ? "#4caf50" : "#9e9e9e", // Green for true, Grey for false
            color: "#fff", // White text
            borderRadius: "4px",
            fontWeight: "bold",
          }}
        />
      </Toolbar>
    </AppBar>
      <Box sx={{ marginTop: 8 }}>
      <Box
        sx={{
          display: "flex",
          flexDirection: { xs: "column", md: "row" },
          gap: 4,
        }}
      >
        <DirectSwapBoard
          tokens={tokensDirectSwap}
          selectedToken={selectedTokenDirectSwap}
          handleTokenChange={handleTokenChangeDirectSwap}
          handleSubmit={handleSubmitDirectSwap}
          targetToken={targetTokenDirectSwap}
          handleTargetToken={handleTargetTokenDirectSwap}
          priorityFee={priorityFeeDirectSwap}
          setPriorityFee={setPriorityFeeDirectSwap}
          slippageTolerance={slippageToleranceDirectSwap}
          setSlippageTolerance={setSlippageToleranceDirectSwap}
          quickSwapValue={quickSwapValueDirectSwap}
          setQuickSwapValue={setQuickSwapValueDirectSwap}
          setWillDoQuickSwap={setWillDoQuickSwap}
          willDoQuickSwap={willDoQuickSwap}
          customAmountDirectSwap={customAmountDirectSwap}
          setCustomAmountDirectSwap={setCustomAmountDirectSwap}
          setSwapPendingStates={setSwapPendingStates}
          swapPendingStates={swapPendingStates}
        />
        {/* <CurrentStateBoard selectedToken={selectedToken} /> */}
        <Box
          sx={{
            display: "flex",
            flexDirection: "column",
            gap: 2,
            flex: 1,
            position: "sticky",
            top: 0,
            zIndex: 10,
          }}
        >
          <SnipeSettingBoard
            tokens={tokensSnipeSetting}
            selectedToken={selectedTokenSnipeSetting}
            handleTokenChange={handleTokenChangeSnipeSetting}
            handleSubmit={handleSubmitSnipeSetting}
            targetToken={targetTokenSnipeSetting}
            handleTargetToken={handleTargetTokenSnipeSetting}
            priorityFee={priorityFeeSnipeSetting}
            setPriorityFee={setPriorityFeeSnipeSetting}
            slippageTolerance={slippageToleranceSnipeSetting}
            setSlippageTolerance={setSlippageToleranceSnipeSetting}
            quickSwapValueSnipeSetting={quickSwapValueSnipeSetting}
            setQuickSwapValueSnipeSetting={setQuickSwapValueSnipeSetting}
            customAmountSnipeSetting={customAmountSnipeSetting}
            setCustomAmountSnipeSetting={setCustomAmountSnipeSetting}
          />
        </Box>
      </Box>
      </Box>

      {/* <Button onClick={() => setSnackbar({ message: "swapping...", vertical: "top", horizontal: "right", open: true })}>Snackbar</Button> */}
      <SnipingBoard
        swapState={swapState}
        isRetrying={isRetrying}
        snipingList={snipingList}
        startSnipe={startSnipe}
        snipe={snipe}
        selectedToken={selectedTokenDirectSwap}
        isSniping={isSniping}
        settings={settings}
        setQuickSwapValue={setQuickSwapValueDirectSwap}
        totalData={totalDataSnipeSetting}
        setTotalData={setTotalDataSnipeSetting}
      />
    </Box>
  );
};

export default TokenSelectorForm;

