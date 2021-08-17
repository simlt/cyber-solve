using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;
using System.Threading.Tasks;
using System.Runtime.InteropServices;
using System.Windows;
using System.Windows.Controls;
using System.Windows.Data;
using System.Windows.Documents;
using System.Windows.Input;
using System.Windows.Interop;
using System.Windows.Media;
using System.Windows.Media.Imaging;
using System.Windows.Navigation;
using System.Windows.Shapes;
using System.Windows.Threading;

namespace overlay
{
    /// <summary>
    /// Interaction logic for MainWindow.xaml
    /// </summary>
    public partial class MainWindow : Window
    {
        private DispatcherTimer dispatcherTimer = new DispatcherTimer();

        public MainWindow()
        {
            InitializeComponent();

            // Parse arguments
            string[] args = Environment.GetCommandLineArgs();
            if (args.Length < 6)
            {
                MessageBox.Show(this, "Usage: overlay.exe img_path left_pos top_pos width height", "Error", MessageBoxButton.OK);
                this.Close();
                return;
            }
            string img_path = args[1];
            double left_pos = double.Parse(args[2]);
            double top_pos = double.Parse(args[3]);
            double width = double.Parse(args[4]);
            double height = double.Parse(args[5]);

            SetOverlayPosition(left_pos, top_pos, width, height);
            LoadImageOverlay(img_path);
            MakeOverlay();

            // Init Timer
            dispatcherTimer.Tick += new EventHandler(Window_KeepOnTop);
            dispatcherTimer.Interval = new TimeSpan(0,0,1);
            dispatcherTimer.Start();
        }

        private void Window_KeepOnTop(object sender, EventArgs e)
        {
            // Reset Topmost to "steal" top again in case another windows set it
            this.Topmost = false;
            this.Topmost = true;
        }

        protected override void OnSourceInitialized(EventArgs e) {
            base.OnSourceInitialized(e);
            this.MakeTransparent();
            this.Show();
        }

        public void LoadImageOverlay(string path)
        {
            if (!System.IO.File.Exists(path))
            {
                var message = String.Format("No such file exists at path {0}", path);
                MessageBox.Show(this, message, "Error", MessageBoxButton.OK);
                this.Close();
                return;
            }

            var pathUri = new Uri(path);
            try
            {
                var img = new BitmapImage(pathUri);
                OverlayImage.Source = img;
            }
            catch (Exception e)
            {
                var message = String.Format("Error while loading image. {0}", e.Message);
                MessageBox.Show(this, message, "Error", MessageBoxButton.OK);
                this.Close();
                return;
            }
        }

        public void SetOverlayPosition(double left, double top, double width, double height)
        {

            this.Left = left;
            this.Top = top;
            this.Width = width;
            this.Height = height;
        }

        public void MakeOverlay() {
            this.WindowStyle = WindowStyle.None;
            this.Background = Brushes.Transparent;
            this.ShowInTaskbar = false;
            this.Topmost = true;
            this.ResizeMode = ResizeMode.NoResize;
            this.AllowsTransparency = true;
        }
    }

    public static class WindowHelper
    {
        const int GWL_EXSTYLE = (-20);
        const int WS_EX_TRANSPARENT = 0x00000020;
        // Make windows transparent and non clickable even on opaque sections
        // https://social.msdn.microsoft.com/Forums/en-US/c32889d3-effa-4b82-b179-76489ffa9f7d/how-to-clicking-throughpassing-shapesellipserectangle?forum=wpf
        public static void MakeTransparent(this Window wnd)
        {
            IntPtr hwnd = new WindowInteropHelper(wnd).Handle;
            int extendedStyle = GetWindowLong(hwnd, GWL_EXSTYLE);
            SetWindowLong(hwnd, GWL_EXSTYLE, extendedStyle | WS_EX_TRANSPARENT);
        }
        [DllImport("user32.dll")]
        static extern int SetWindowLong(IntPtr hwnd, int index, int newStyle);
        [DllImport("user32.dll")]
        static extern int GetWindowLong(IntPtr hwnd, int index);
    }
}
