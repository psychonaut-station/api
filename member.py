#!/bin/env python3

import sys
from selenium import webdriver

driver = webdriver.Remote("http://localhost:4444/wd/hub", options = webdriver.ChromeOptions())
driver.set_window_size(1280, 1024)
driver.get(f"https://secure.byond.com/members/{sys.argv[1]}?format=text")

#print(driver.page_source.encode("utf-8"))
html = driver.page_source.encode("utf-8")

driver.quit()

if "ckey = " in str(html):
    print("true")
else:
    print("false")
