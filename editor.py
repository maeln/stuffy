import sys
import random

from ctypes import cdll, c_double
from sys import platform

from PySide2.QtGui import QWindow, QOpenGLContext, QSurface, QSurfaceFormat, QExposeEvent
from PySide2.QtWidgets import QApplication
from PySide2.QtCore import QSize, QEvent

if platform == 'darwin':
    prefix = 'lib'
    ext = 'dylib'
elif platform == 'win32':
    prefix = ''
    ext = 'dll'
else:
    prefix = 'lib'
    ext = 'so'

lib_path = 'target/debug/{}peglrs.{}'.format(prefix, ext)
print(lib_path)
lib = cdll.LoadLibrary(lib_path)

load_gl_symbol = lib.load_gl_symbol
init_gl = lib.init_gl
init_scene = lib.init_scene
display_loop = lib.display_loop
print_gl_info = lib.print_gl_info
resize_window = lib.resize_window


class GLWin(QWindow):
    def __init__(self):
        QWindow.__init__(self)
        self.setSurfaceType(QSurface.OpenGLSurface)
        self.gl_format = QSurfaceFormat()
        self.gl_format.setRenderableType(QSurfaceFormat.OpenGL)
        self.gl_format.setProfile(QSurfaceFormat.CoreProfile)
        self.gl_format.setVersion(4, 1)
        self.setFormat(self.gl_format)
        if self.supportsOpenGL():
            print("OpenGL supported !")

    def init_context(self):
        self.gl_context = QOpenGLContext(self)
        self.gl_context.setFormat(self.gl_format)
        if self.gl_context.create():
            print("Context created !")
        if self.gl_context.makeCurrent(self):
            print("Context made current !")

    def init_scene(self):
        width = c_double(self.size().width())
        height = c_double(self.size().height())
        dpi_ratio = c_double(self.devicePixelRatio())

        load_gl_symbol()
        init_gl(width, height, dpi_ratio)
        print_gl_info()
        init_scene(width, height, dpi_ratio)

    def render_scene(self):
        if self.isExposed():
            display_loop(c_double(0.0))
            self.gl_context.swapBuffers(self)

    def resize(self):
        if self.isExposed():
            width = c_double(self.size().width())
            height = c_double(self.size().height())
            dpi_ratio = c_double(self.devicePixelRatio())
            resize_window(width, height, dpi_ratio)

    def event(self, ev):
        if ev.type == QEvent.UpdateRequest:
            self.render_scene()
            return True
        else:
            return super().event(ev)

    def exposeEvent(self, ev):
        self.render_scene()

    def resizeEvent(self, ev):
        self.resize()
        self.render_scene()


if __name__ == "__main__":
    app = QApplication(sys.argv)

    win = GLWin()
    win.setBaseSize(QSize(640, 480))
    win.show()
    win.init_context()
    win.init_scene()

    sys.exit(app.exec_())
