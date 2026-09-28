from pathlib import Path
import sys
from unittest import TestCase, mock

sys.path.insert(0, str(Path(__file__).resolve().parents[2] / "tools"))
import osc_client


class HandshakeTests(TestCase):
    def client(self):
        client = osc_client.OscClient.__new__(osc_client.OscClient)
        client.socket = mock.Mock()
        client.socket.getsockname.return_value = ("127.0.0.1", 12345)
        client.send = mock.Mock()
        return client

    def reset(self):
        error = ConnectionResetError("port not listening yet")
        error.winerror = 10054
        return error

    def test_windows_startup_icmp_is_retried_until_ready(self):
        client = self.client()
        ready = [1, "0.0.16", "Dx12", "NVIDIA"]
        client.expect = mock.Mock(side_effect=[self.reset(), ready])
        with mock.patch.object(osc_client.os, "name", "nt"):
            self.assertEqual(client.hello(), ready)
        self.assertEqual(client.send.call_count, 2)

    def test_other_socket_errors_are_not_hidden(self):
        client = self.client()
        client.expect = mock.Mock(side_effect=PermissionError("blocked"))
        with self.assertRaises(PermissionError):
            client.hello()

    def test_startup_retry_remains_bounded(self):
        client = self.client()
        client.expect = mock.Mock(side_effect=self.reset())
        with mock.patch.object(osc_client.os, "name", "nt"):
            with self.assertRaises(TimeoutError):
                client.hello(timeout=0.02)
