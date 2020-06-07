from lxml import etree
from io import StringIO
from flask import Flask, request

app = Flask(__name__)


@app.route("/xpath1")
def xpath1():
    xpathQuery = request.args.get('xml', '')
    f = StringIO('<foo><bar></bar></foo>')
    tree = etree.parse(f)
    r = tree.xpath(xpathQuery)


@app.route("/xpath2")
def xpath2():
    xpathQuery = request.args.get('xml', '')

    root = etree.XML("<root><a>TEXT</a></root>")
    find_text = etree.XPath(xpathQuery)
    text = find_text(root)[0]


@app.route("/xpath3")
def xpath3():
    xpathQuery = request.args.get('xml', '')
    root = etree.XML("<root><a>TEXT</a></root>")
    find_text = etree.XPath(xpathQuery, smart_strings=False)
    text = find_text(root)[0]


@app.route("/xpath4")
def xpath4():
    xpathQuery = request.args.get('xml', '')
    root = etree.XML("<root><a>TEXT</a></root>")
    find_text = etree.ETXPath(xpathQuery)
    text = find_text(root)[0]
