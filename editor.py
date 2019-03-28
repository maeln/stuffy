import sys
import random

from ctypes import cdll, c_double, c_float, c_uint
from sys import platform

from PySide2.QtGui import QWindow, QOpenGLContext, QSurface, QSurfaceFormat, QExposeEvent
from PySide2.QtWidgets import QApplication, QOpenGLWidget
from PySide2.QtCore import QSize, QEvent, Signal, Slot, Qt

if platform == 'darwin':
    prefix = 'lib'
    ext = 'dylib'
elif platform == 'win32':
    prefix = ''
    ext = 'dll'
else:
    prefix = 'lib'
    ext = 'so'

lib_path = './target/debug/{}peglrs.{}'.format(prefix, ext)

print(lib_path)
lib = cdll.LoadLibrary(lib_path)

load_gl_symbol = lib.load_gl_symbol
init_gl = lib.init_gl
init_scene = lib.init_scene
display_loop = lib.display_loop
print_gl_info = lib.print_gl_info
resize_window = lib.resize_window
handle_mouse = lib.handle_mouse


class GLWidget(QOpenGLWidget):
    def __init__(self, parent=None):
        QOpenGLWidget.__init__(self, parent)
        self.gl_format = QSurfaceFormat()
        self.gl_format.setRenderableType(QSurfaceFormat.OpenGL)
        self.gl_format.setProfile(QSurfaceFormat.CoreProfile)
        self.gl_format.setVersion(4, 1)
        self.setFormat(self.gl_format)
        self.mouse_x = 0
        self.mouse_y = 0
        self.mouse_init = False
        self.mouse_pressed = False

    def paintGL(self):
        display_loop(c_double(0.0), c_uint(self.defaultFramebufferObject()))

    def resizeGL(self, width, height):
        width = c_double(self.size().width())
        height = c_double(self.size().height())
        dpi_ratio = c_double(self.devicePixelRatio())
        resize_window(width, height, dpi_ratio)

    def initializeGL(self):
        width = c_double(self.size().width())
        height = c_double(self.size().height())
        dpi_ratio = c_double(self.devicePixelRatio())
        load_gl_symbol()
        init_gl(width, height, dpi_ratio)
        print_gl_info()
        init_scene(width, height, dpi_ratio)

    def mousePressEvent(self, ev):
        if ev.button() == Qt.LeftButton:
            self.mouse_pressed = True

    def mouseReleaseEvent(self, ev):
        if ev.button() == Qt.LeftButton:
            self.mouse_pressed = False
            self.mouse_init = False

    def mouseMoveEvent(self, ev):
        pos = ev.localPos()
        if self.mouse_pressed:
            if not self.mouse_init:
                self.mouse_x = pos.x()
                self.mouse_y = pos.y()
                self.mouse_init = True
            else:
                dx = self.mouse_x - pos.x()
                dy = self.mouse_y - pos.y()
                self.mouse_x = pos.x()
                self.mouse_y = pos.y()
                handle_mouse(c_float(dx), c_float(dy), c_float(0.001))
                self.update()


class GLWin(QWindow):
    requestRender = Signal()

    def __init__(self, parent=None):
        QWindow.__init__(self, parent)
        self.setSurfaceType(QSurface.OpenGLSurface)
        self.gl_format = QSurfaceFormat()
        self.gl_format.setRenderableType(QSurfaceFormat.OpenGL)
        self.gl_format.setProfile(QSurfaceFormat.CoreProfile)
        self.gl_format.setVersion(4, 1)
        self.setFormat(self.gl_format)
        if self.supportsOpenGL():
            print("OpenGL supported !")
        self.mouse_x = 0
        self.mouse_y = 0
        self.mouse_init = False
        self.animating = False
        self.requestRender.connect(self.requestUpdate)
        # self.setMouseGrabEnabled(True)
        self.mouse_pressed = False

    def mousePressEvent(self, ev):
        if ev.button() == Qt.LeftButton:
            self.mouse_pressed = True

    def mouseReleaseEvent(self, ev):
        if ev.button() == Qt.LeftButton:
            self.mouse_pressed = False
            self.mouse_init = False

    def mouseMoveEvent(self, ev):
        pos = ev.localPos()
        if self.mouse_pressed:
            if not self.mouse_init:
                self.mouse_x = pos.x()
                self.mouse_y = pos.y()
                self.mouse_init = True
            else:
                dx = self.mouse_x - pos.x()
                dy = self.mouse_y - pos.y()
                self.mouse_x = pos.x()
                self.mouse_y = pos.y()
                handle_mouse(c_float(dx), c_float(dy), c_float(0.001))

    def start(self):
        self.animating = True
        self.renderLater()

    def render(self):
        display_loop(c_double(0.0))

    def renderLater(self):
        self.requestRender.emit()

    def renderNow(self):
        if not self.isExposed():
            return
        self.render()
        self.gl_context.swapBuffers(self)
        if self.animating:
            self.renderLater()

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

    def resize(self):
        if self.isExposed():
            width = c_double(self.size().width())
            height = c_double(self.size().height())
            dpi_ratio = c_double(self.devicePixelRatio())
            resize_window(width, height, dpi_ratio)

    def event(self, ev):
        if ev.type() == QEvent.UpdateRequest:
            self.renderNow()
            return True
        else:
            return super().event(ev)

    def exposeEvent(self, ev):
        self.renderLater()

    def resizeEvent(self, ev):
        self.resize()
        self.renderLater()


if __name__ == "__main__":
    app = QApplication(sys.argv)

    #win = GLWin()
    #win.setBaseSize(QSize(640, 480))
    # win.show()
    # win.init_context()
    # win.init_scene()
    # win.start()

    win = GLWidget()
    win.setBaseSize(QSize(640, 480))
    win.show()

    sys.exit(app.exec_())
