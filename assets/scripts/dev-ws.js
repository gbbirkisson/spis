// This is a script that opens up a websocket to the server, when that websocket looses
// connection we will try to reconnect to the server. Once we are able to reconnect to the
// server, we refresh the page.

var socket_was_connected = false;

function connect() {
  var socket = new WebSocket(((window.location.protocol === "https:") ? "wss://" : "ws://") + window.location.host + "/dev/ws");

  socket.onopen = function(_event) {
    if (socket_was_connected) {
      location.reload();
    } else {
      socket_was_connected = true;
    }
  }

  socket.onclose = function(_event) {
    // Allow the last socket to be cleaned up.
    socket = null;

    // Set an interval to continue trying to reconnect
    // periodically until we succeed.
    setTimeout(function() {
      connect();
    }, 1000)
  }
}

connect();
